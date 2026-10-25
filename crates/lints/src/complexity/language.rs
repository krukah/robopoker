use std::ops::Range;

use super::Unit;
use super::branch::Branch;
use crate::*;

pub const MAX_COMPLEXITY: usize = 15;

pub const RULE: &str = "branch-complexity";

const RUST_UNITS: &[&str] = &["function_item", "closure_expression"];

const TS_UNITS: &[&str] = &[
    "function_declaration",
    "generator_function_declaration",
    "function_expression",
    "generator_function",
    "arrow_function",
    "method_definition",
];

impl Language {
    /// Scores every unit in `src`, reporting only the ones a diff reaches.
    /// Pass `None` to score the whole file.
    pub fn complexity(self, src: &str, changed: Option<&[Range<usize>]>) -> Vec<Violation> {
        self.units(src)
            .into_iter()
            .filter(Unit::over_limit)
            .filter(|unit| unit.touched(changed))
            .map(Violation::from)
            .collect()
    }

    pub fn units(self, src: &str) -> Vec<Unit> {
        self.parse(src)
            .map(|tree| Branch::new(tree.root_node(), src, self).units())
            .map(Unit::by_line)
            .unwrap_or_default()
    }

    pub(super) fn unit_kinds(self) -> &'static [&'static str] {
        match self {
            Self::Rust => RUST_UNITS,
            Self::TypeScript | Self::Tsx => TS_UNITS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Language {
        fn score(self, src: &str) -> usize {
            self.units(src).first().map(Unit::score).expect("one unit")
        }

        fn scores(self, src: &str) -> Vec<usize> {
            self.units(src).iter().map(Unit::score).collect()
        }

        fn names(self, src: &str) -> Vec<String> {
            self.units(src).iter().map(Unit::name).map(str::to_string).collect()
        }
    }

    #[test]
    fn a_comment_between_arms_does_not_hide_one() {
        let bare = "fn f(x: A) -> u8 { match x { A::P if p() => 1, A::Q if q() => 2, _ => 3 } }";
        let noted = "fn f(x: A) -> u8 { match x {\n A::P if p() => 1, // note\n A::Q if q() => 2, // note\n _ => 3 } }";
        assert_eq!(Language::Rust.score(bare), Language::Rust.score(noted));
    }

    #[test]
    fn a_straight_line_function_scores_one() {
        assert_eq!(Language::Rust.score("fn f() { let a = 1; g(a); }"), 1);
    }

    #[test]
    fn an_if_adds_one_and_a_bare_else_adds_none() {
        assert_eq!(Language::Rust.score("fn f() { if a { b() } else { c() } }"), 2);
    }

    #[test]
    fn an_else_if_chain_counts_each_test() {
        assert_eq!(Language::Rust.score("fn f() { if a { x() } else if b { y() } else { z() } }"), 3);
    }

    #[test]
    fn an_exhaustive_lookup_match_scores_one() {
        assert_eq!(Language::Rust.score("fn f() { match a { 1 => x(), 2 => y(), _ => z() } }"), 1);
        let wide = (0..200).map(|i| format!("{i} => v{i},")).collect::<Vec<_>>().join(" ");
        assert_eq!(Language::Rust.score(&format!("fn f() {{ match a {{ {wide} }} }}")), 1);
    }

    #[test]
    fn an_arm_that_branches_counts_itself_and_its_branch() {
        assert_eq!(
            Language::Rust.score("fn f() { match a { 1 => x(), 2 => if b { y() } else { z() }, _ => w() } }"),
            3
        );
    }

    #[test]
    fn a_match_returning_a_nested_lookup_stays_flat() {
        let src = "fn f() { match a { 1 => match b { 2 => x(), _ => y() }, _ => z() } }";
        assert_eq!(Language::Rust.score(src), 1);
    }

    #[test]
    fn a_match_guard_makes_an_arm_branch() {
        assert_eq!(Language::Rust.score("fn f() { match a { n if n > 0 => x(), _ => z() } }"), 2);
        assert_eq!(Language::Rust.score("fn f() { match a { 1 => x(), n if n > 0 => y(), _ => z() } }"), 3);
    }

    #[test]
    fn logical_operators_count_but_arithmetic_does_not() {
        assert_eq!(Language::Rust.score("fn f() { if a && b || c { x() } }"), 4);
        assert_eq!(Language::Rust.score("fn f() { let n = a + b * c; }"), 1);
    }

    #[test]
    fn loops_count() {
        assert_eq!(Language::Rust.score("fn f() { for a in b { } while c { } loop { break } }"), 4);
    }

    #[test]
    fn a_let_else_counts() {
        assert_eq!(Language::Rust.score("fn f() { let Some(a) = b else { return }; }"), 2);
    }

    #[test]
    fn an_if_let_counts_once() {
        assert_eq!(Language::Rust.score("fn f() { if let Some(a) = b { x(a) } }"), 2);
    }

    #[test]
    fn a_let_chain_counts_each_conjunct() {
        assert_eq!(Language::Rust.score("fn f() { if let Some(a) = b && let Some(c) = d { x() } }"), 3);
    }

    #[test]
    fn the_question_mark_is_free() {
        assert_eq!(Language::Rust.score("fn f() -> R { a()?; b()?; c()?; Ok(()) }"), 1);
    }

    #[test]
    fn a_closure_scores_apart_from_its_function() {
        let src = "fn f() { if a { x() } let g = |n| if n { 1 } else { 2 }; }";
        assert_eq!(Language::Rust.scores(src), vec![2, 2]);
    }

    #[test]
    fn a_nested_function_scores_apart() {
        let src = "fn outer() { if a { x() } fn inner() { if b { y() } } }";
        assert_eq!(Language::Rust.scores(src), vec![2, 2]);
    }

    #[test]
    fn a_bound_closure_reports_its_binding_name() {
        assert_eq!(Language::TypeScript.names("const Screen = () => { return 1 }"), vec!["`Screen`"]);
        assert_eq!(Language::Rust.names("fn f() { let g = |n| n; }"), vec!["`f`", "`g`"]);
        assert_eq!(Language::Rust.names("fn f() { call(|n| n); }"), vec!["`f`", "a closure in `f`"]);
    }

    #[test]
    fn typescript_forgives_a_lookup_switch_and_counts_a_branching_one() {
        let lookup = "function f(a) { switch (a) { case 1: return 1; default: return 3; } }";
        assert_eq!(Language::TypeScript.scores(lookup), vec![1]);
        let branching = "function f(a) { switch (a) { case 1: return b ? 1 : 2; default: return 3; } }";
        assert_eq!(Language::TypeScript.scores(branching), vec![3]);
    }

    #[test]
    fn typescript_counts_ternaries_nullish_coalescing_and_catch() {
        assert_eq!(Language::TypeScript.scores("function f(a) { return a ? 1 : 2; }"), vec![2]);
        assert_eq!(Language::TypeScript.scores("function f(a) { try { g(a ?? 1) } catch (e) { h(e) } }"), vec![3]);
    }

    #[test]
    fn a_directive_above_the_signature_exempts_the_function() {
        let branches = (0..20)
            .map(|i| format!("if a{i} {{ x() }}"))
            .collect::<Vec<_>>()
            .join(" ");
        let src = format!("fn f() {{ {branches} }}");
        assert_eq!(Language::Rust.complexity(&src, None).len(), 1);
        let allowed = format!("// lint:allow(branch-complexity) dispatch table\n{src}");
        assert!(Language::Rust.complexity(&allowed, None).is_empty());
        let attributed = format!("// lint:allow(branch-complexity) dispatch table\n#[inline]\n{src}");
        assert!(Language::Rust.complexity(&attributed, None).is_empty());
    }

    #[test]
    fn an_untouched_function_is_not_reported() {
        let branches = (0..20)
            .map(|i| format!("if a{i} {{ x() }}"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = format!("fn f() {{\n{branches}\n}}\nfn g() {{}}\n");
        let elsewhere = [23..24, 24..25];
        assert!(Language::Rust.complexity(&src, Some(&elsewhere)).is_empty());
        let inside = [5..6, 23..24];
        assert_eq!(Language::Rust.complexity(&src, Some(&inside)).len(), 1);
    }

    #[test]
    fn a_trait_signature_has_no_body_and_no_score() {
        assert!(Language::Rust.units("trait T { fn f(&self); }").is_empty());
    }
}
