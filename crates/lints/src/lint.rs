use std::collections::HashSet;
use std::ops::Range;

use crate::*;

impl Language {
    pub fn lint_source(self, src: &str) -> Vec<Violation> {
        self.lint_source_ranges(src, None)
    }

    pub fn lint_source_ranges(self, src: &str, changed: Option<&[Range<usize>]>) -> Vec<Violation> {
        self.lint_file("", src, changed)
    }

    pub fn lint_file(self, path: &str, src: &str, changed: Option<&[Range<usize>]>) -> Vec<Violation> {
        let mut violations = self.lint_blocks(src, changed);
        violations.extend(self.complexity(src, changed));
        violations.extend(self.shape(path, src));
        violations.extend(Entry::check(path, src));
        violations.extend(self.check_doc_code_ratio(src));
        violations.sort_by_key(|v| (v.line(), v.rule()));
        violations
    }

    fn lint_blocks(self, src: &str, changed: Option<&[Range<usize>]>) -> Vec<Violation> {
        let blocks = Block::collect(src, self);
        let directives = blocks.iter().map(Directive::parse).collect::<Vec<Option<Directive>>>();
        let mut violations = Vec::new();
        for (index, block) in blocks.iter().enumerate() {
            if let Some(ranges) = changed {
                let touched = ranges
                    .iter()
                    .any(|Range { start, end }| *start <= block.end_line() && block.start_line() < *end);
                if !touched {
                    continue;
                }
            }
            let carried = Block::carried_allows(index, &blocks, &directives);
            block.check(directives[index].as_ref(), &carried, self, &mut violations);
        }
        violations
    }

    fn check_doc_code_ratio(self, src: &str) -> Option<Violation> {
        let blocks = Block::collect(src, self);
        if blocks.iter().any(|block| block.allows_rule("doc-code-ratio")) {
            return None;
        }
        let doc = blocks
            .iter()
            .filter(|block| block.doc() && !block.member_doc())
            .map(|block| block.content().iter().filter(|line| !line.trim().is_empty()).count())
            .sum::<usize>();
        let covered = blocks
            .into_iter()
            .filter(Block::own_line)
            .flat_map(|block| block.start_line()..=block.end_line())
            .collect::<HashSet<_>>();
        let code = src
            .lines()
            .enumerate()
            .filter(|(index, line)| !line.trim().is_empty() && !covered.contains(&(index + 1)))
            .count();
        if code < MIN_RATIO_CODE_LINES {
            return None;
        }
        let ratio = doc as f64 / code as f64;
        (ratio > MAX_DOC_CODE_RATIO).then(|| Violation::new(1, 1, "doc-code-ratio", format!(
                "{doc} doc lines to {code} code lines ({ratio:.2}); the maximum is {MAX_DOC_CODE_RATIO:.2}. Cut prose that restates a signature"
            )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Language {
        /// These fixtures exercise the comment rules alone; `shape::language`
        /// owns the shape rules, including the `free-function` a bare `fn` trips.
        fn comment_rules(self, src: &str) -> Vec<&'static str> {
            self.lint_file("a.rs", src, None)
                .into_iter()
                .map(|v| v.rule())
                .filter(|rule| !OPT_IN_RULES.contains(rule))
                .filter(|rule| *rule != shape::FREE_FUNCTION)
                .collect()
        }

        fn source_rules(self, src: &str) -> Vec<&'static str> {
            self.lint_source(src).into_iter().map(|v| v.rule()).collect()
        }

        fn ratio_rules(self, src: &str) -> Vec<&'static str> {
            self.lint_file("a_test.rs", src, None)
                .into_iter()
                .map(|v| v.rule())
                .collect()
        }
    }

    #[test]
    fn two_line_comment_passes() {
        let src = "// the reaper runs first\n// then the heartbeat check\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn three_line_comment_fails() {
        let src = "// one\n// two\n// three\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["long-comment"]);
    }

    #[test]
    fn three_line_block_comment_fails() {
        let src = "/* one\ntwo\nthree */\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["long-comment"]);
    }

    #[test]
    fn blocks_split_by_blank_line_pass() {
        let src = "// one\n\n// two\n// three\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn trailing_comment_does_not_merge_with_next_line() {
        let src = "fn main() {\n    let a = 1; // one\n    // two\n    // three\n}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn allow_directive_suppresses_long_comment() {
        let src = "// lint:allow(long-comment) invariant needs full context\n// one\n// two\n// three\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn allow_directive_requires_reason() {
        let src = "// lint:allow(long-comment)\n// one\n// two\n// three\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["invalid-allow"]);
    }

    #[test]
    fn allow_directive_rejects_unknown_rule() {
        let src = "// lint:allow(nonsense) reason here\n// one\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["invalid-allow"]);
    }

    #[test]
    fn allow_directive_takes_multiple_rules() {
        let src = "enum E {\n    // lint:allow(type-comment, todo-comment) tracked in the variant docs\n    // TODO stays here\n    A,\n}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn short_doc_comments_pass() {
        let src = "/// one\n/// two\n/// three\n/// four\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
        let inner = "//! one\n//! two\n//! three\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(inner).is_empty());
    }

    #[test]
    fn five_content_line_doc_comment_passes() {
        let src = "/// a\n/// b\n/// c\n/// d\n/// e\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn six_content_line_doc_comment_fails() {
        let src = "/// a\n/// b\n/// c\n/// d\n/// e\n/// f\npub fn f() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["long-doc-comment"]);
        let inner = "//! a\n//! b\n//! c\n//! d\n//! e\n//! f\npub fn f() {}\n";
        assert_eq!(Language::Rust.comment_rules(inner), vec!["long-doc-comment"]);
    }

    #[test]
    fn blank_doc_lines_do_not_count() {
        let src = "/// a\n///\n/// b\n/// c\n/// d\n/// e\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn a_doc_comment_on_a_test_item_is_held_to_the_comment_cap() {
        let in_mod = "#[cfg(test)]\nmod tests {\n    /// a\n    /// b\n    /// c\n    #[test]\n    fn t() {}\n}\n";
        assert_eq!(Language::Rust.comment_rules(in_mod), vec!["hidden-doc-comment"]);
        let two_lines = "#[cfg(test)]\nmod tests {\n    /// a\n    /// b\n    #[test]\n    fn t() {}\n}\n";
        assert!(Language::Rust.comment_rules(two_lines).is_empty());
    }

    #[test]
    fn a_doc_comment_on_an_intg_test_item_is_held_to_the_comment_cap() {
        let three_lines = "/// a\n/// b\n/// c\n#[intg_test]\nasync fn t() {}\n";
        assert_eq!(Language::Rust.comment_rules(three_lines), vec!["hidden-doc-comment"]);
        let two_lines = "/// a\n/// b\n#[intg_test]\nasync fn t() {}\n";
        assert!(Language::Rust.comment_rules(two_lines).is_empty());
    }

    #[test]
    fn a_doc_comment_on_a_private_field_is_held_to_the_comment_cap() {
        let src = "pub struct S {\n    /// a\n    /// b\n    /// c\n    hidden: u8,\n}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["hidden-doc-comment"]);
    }

    #[test]
    fn a_private_function_keeps_the_doc_cap() {
        let five = "/// a\n/// b\n/// c\n/// d\n/// e\nfn f() {}\n";
        assert!(Language::Rust.comment_rules(five).is_empty());
        let six = "/// a\n/// b\n/// c\n/// d\n/// e\n/// f\nfn f() {}\n";
        assert_eq!(Language::Rust.comment_rules(six), vec!["hidden-doc-comment"]);
        let public = "/// a\n/// b\n/// c\n/// d\n/// e\n/// f\npub fn f() {}\n";
        assert_eq!(Language::Rust.comment_rules(public), vec!["long-doc-comment"]);
    }

    #[test]
    fn a_trait_impl_member_is_not_hidden() {
        let src = "pub struct S;\nimpl std::fmt::Debug for S {\n    /// a\n    /// b\n    /// c\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }\n}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn prose_heavier_than_logic_fails_the_file() {
        let body = (0..40)
            .map(|i| format!("/// prose {i}\npub fn f{i}() {{}}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(Language::Rust.ratio_rules(&body).contains(&"doc-code-ratio"));
    }

    #[test]
    fn field_and_variant_docs_do_not_count_toward_the_ratio() {
        let fields = (0..40)
            .map(|i| format!("    /// what {i} holds\n    pub f{i}: u8,"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = format!("pub struct S {{\n{fields}\n}}\n");
        assert!(!Language::Rust.ratio_rules(&src).contains(&"doc-code-ratio"));
    }

    #[test]
    fn a_small_file_is_exempt_from_the_ratio() {
        let src = "/// a\n/// b\n/// c\n/// d\npub fn f() {}\n";
        assert!(!Language::Rust.ratio_rules(src).contains(&"doc-code-ratio"));
    }

    #[test]
    fn a_file_directive_suppresses_the_ratio() {
        let body = (0..40)
            .map(|i| format!("/// prose {i}\npub fn f{i}() {{}}"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = format!("// lint:allow(doc-code-ratio) published api surface\n{body}");
        assert!(!Language::Rust.ratio_rules(&src).contains(&"doc-code-ratio"));
    }

    #[test]
    fn doc_comment_content_rules_apply() {
        let src = "/// see ENG-4122, TODO later\npub fn f() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["ticket-reference", "todo-comment"]);
    }

    #[test]
    fn a_doctest_is_not_commented_out_code() {
        let src = "/// ```\n/// let x = f();\n/// ```\npub fn f() -> u8 { 1 }\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn a_doc_comment_still_escapes_the_structural_rules() {
        let long = "/// a\n/// b\n/// c\npub fn f() {}\n";
        assert!(Language::Rust.comment_rules(long).is_empty());
        let in_struct = "pub struct S {\n    /// what it holds\n    a: u8,\n}\n";
        assert!(Language::Rust.comment_rules(in_struct).is_empty());
    }

    #[test]
    fn doc_directive_first_line_suppresses() {
        let src = "/// lint:allow(long-doc-comment) public api contract\n/// a\n/// b\n/// c\n/// d\n/// e\n/// f\npub fn f() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn directive_above_doc_comment_suppresses() {
        let src = "// lint:allow(long-doc-comment) public api contract\n/// a\n/// b\n/// c\n/// d\n/// e\n/// f\npub fn f() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn directive_with_gap_does_not_carry() {
        let src = "// lint:allow(long-doc-comment) public api contract\n\n/// a\n/// b\n/// c\n/// d\n/// e\n/// f\npub fn f() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["long-doc-comment"]);
    }

    #[test]
    fn directive_above_trailing_enum_comment_carries() {
        let src =
            "enum E {\n    // lint:allow(type-comment) the wire value must stay visible\n    A, // wire value 3\n}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn doc_comment_on_enum_variant_passes() {
        let src = "enum E {\n    /// documented variant\n    A,\n}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn comment_in_enum_fails() {
        let src = "enum E {\n    // grouping note\n    A,\n    B,\n}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn trailing_comment_in_enum_fails() {
        let src = "enum E {\n    A, // trailing note\n}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn comment_in_struct_fails() {
        let src = "struct S {\n    // section\n    a: u32,\n}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn comment_in_union_fails() {
        let src = "union U {\n    // raw view\n    a: u32,\n}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn comment_in_trait_fails() {
        let src = "trait T {\n    // lifecycle\n    fn f(&self);\n}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn comment_between_impl_items_fails() {
        let src = "struct S;\nimpl S {\n    fn a(&self) {}\n    // helpers\n    fn b(&self) {}\n}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn comment_between_match_arms_fails() {
        let src = "fn f(x: u32) -> u32 {\n    match x {\n        // fallthrough\n        _ => 0,\n    }\n}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn comment_inside_fn_body_passes() {
        let src = "struct S;\nimpl S {\n    fn a(&self) {\n        // fine here\n    }\n}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn comment_inside_match_arm_block_passes() {
        let src = "fn f(x: u32) -> u32 {\n    match x {\n        _ => {\n            // fine here\n            0\n        }\n    }\n}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn comment_inside_mod_body_passes() {
        let src = "mod m {\n    // module scoped note\n    pub fn f() {}\n}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn ticket_reference_fails() {
        let src = "// see ENG-4122 for the fee logic\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["ticket-reference"]);
    }

    #[test]
    fn pr_number_fails() {
        let src = "// fixed in #12345\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["ticket-reference"]);
    }

    #[test]
    fn linear_url_fails() {
        let src = "// context: linear.app/acme/issue/INF-1
fn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["ticket-reference"]);
    }

    #[test]
    fn technical_tokens_pass() {
        let src = "// decode UTF-8, then hash with SHA-256\n// timestamps use ISO-8601\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn commented_code_single_line_fails() {
        let src = "// let x = compute();\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["commented-code"]);
    }

    #[test]
    fn commented_code_item_fails() {
        let src = "// fn old() {\n//     next();\n// }\nfn next() {}\n";
        let found = Language::Rust.comment_rules(src);
        assert!(found.contains(&"commented-code"), "{found:?}");
        assert!(found.contains(&"long-comment"), "{found:?}");
    }

    #[test]
    fn prose_passes() {
        let src = "// the reaper must run before the heartbeat\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn prose_with_punctuation_passes() {
        let src = "// callers hold the lock; drop order matters\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn todo_fails() {
        let src = "// TODO clean this up\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["todo-comment"]);
        let lower = "// fixme: later\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(lower), vec!["todo-comment"]);
    }

    #[test]
    fn hack_prose_passes() {
        let src = "// this hacky workaround stays until the redesign\nfn main() {}\n";
        assert!(Language::Rust.comment_rules(src).is_empty());
    }

    #[test]
    fn banner_fails() {
        let src = "// ----------\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(src), vec!["banner-comment"]);
        let wrapped = "// ==== HELPERS ====\nfn main() {}\n";
        assert_eq!(Language::Rust.comment_rules(wrapped), vec!["banner-comment"]);
    }

    #[test]
    fn changed_ranges_filter_blocks() {
        let src = "// one\n// two\n// three\nfn main() {}\n";
        assert!(Language::Rust.lint_source_ranges(src, Some(&[5..6, 10..12])).is_empty());
        assert_eq!(Language::Rust.lint_source_ranges(src, Some(&[2..3, 10..12])).len(), 1);
    }

    #[test]
    fn language_for_path_dispatches() {
        assert_eq!(Language::for_path("rust/src/lib.rs"), Some(Language::Rust));
        assert_eq!(Language::for_path("nest/apps/api/main.ts"), Some(Language::TypeScript));
        assert_eq!(Language::for_path("mobile-app/App.tsx"), Some(Language::Tsx));
        assert_eq!(Language::for_path("shared/types.d.ts"), None);
        assert_eq!(Language::for_path("scripts/run.sh"), None);
    }

    #[test]
    fn ts_three_line_comment_fails() {
        let src = "// one\n// two\n// three\nconst x = 1;\n";
        assert_eq!(Language::TypeScript.source_rules(src), vec!["long-comment"]);
    }

    #[test]
    fn ts_short_jsdoc_passes() {
        let src = "/**\n * Long JSDoc block.\n * With several lines.\n * @param x the input\n */\nfunction f(x: number): number {\n  return x;\n}\n";
        assert!(Language::TypeScript.source_rules(src).is_empty());
    }

    #[test]
    fn ts_jsdoc_over_five_content_lines_fails() {
        let src = "/**\n * a\n * b\n * c\n * d\n * e\n * f\n */\nconst x = 1;\n";
        assert_eq!(Language::TypeScript.source_rules(src), vec!["long-doc-comment"]);
    }

    #[test]
    fn ts_directive_above_jsdoc_suppresses() {
        let src = "// lint:allow(long-doc-comment) public api contract\n/**\n * a\n * b\n * c\n * d\n * e\n * f\n */\nconst x = 1;\n";
        assert!(Language::TypeScript.source_rules(src).is_empty());
    }

    #[test]
    fn ts_jsdoc_in_interface_passes() {
        let src = "interface I {\n  /** documented field */\n  x: number;\n}\n";
        assert!(Language::TypeScript.source_rules(src).is_empty());
    }

    #[test]
    fn ts_comment_in_enum_fails() {
        let src = "enum E {\n  // grouping note\n  A,\n  B,\n}\n";
        assert_eq!(Language::TypeScript.source_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn ts_comment_in_interface_fails() {
        let src = "interface I {\n  // section\n  x: number;\n}\n";
        assert_eq!(Language::TypeScript.source_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn ts_comment_in_type_literal_fails() {
        let src = "type T = {\n  // section\n  y: string;\n};\n";
        assert_eq!(Language::TypeScript.source_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn ts_comment_between_class_members_fails() {
        let src = "class C {\n  a(): void {}\n  // helpers\n  b(): void {}\n}\n";
        assert_eq!(Language::TypeScript.source_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn ts_comment_between_switch_cases_fails() {
        let src = "function f(x: number): number {\n  switch (x) {\n    // fallthrough\n    case 1:\n      return 1;\n    default:\n      return 0;\n  }\n}\n";
        assert_eq!(Language::TypeScript.source_rules(src), vec!["type-comment"]);
    }

    #[test]
    fn ts_comment_inside_method_body_passes() {
        let src = "class C {\n  m(): void {\n    // fine here\n  }\n}\n";
        assert!(Language::TypeScript.source_rules(src).is_empty());
    }

    #[test]
    fn ts_comment_inside_case_statements_passes() {
        let src = "function f(x: number): number {\n  switch (x) {\n    case 1:\n      // fine here\n      return 1;\n    default:\n      return 0;\n  }\n}\n";
        assert!(Language::TypeScript.source_rules(src).is_empty());
    }

    #[test]
    fn ts_commented_code_fails() {
        let src = "// const x = compute();\nconst y = 1;\n";
        assert_eq!(Language::TypeScript.source_rules(src), vec!["commented-code"]);
        let call = "// refreshTokens(pool);\nconst y = 1;\n";
        assert_eq!(Language::TypeScript.source_rules(call), vec!["commented-code"]);
    }

    #[test]
    fn ts_prose_with_commas_and_semicolon_passes() {
        let src = "// cleanup, then retry;\nconst y = 1;\n";
        assert!(Language::TypeScript.source_rules(src).is_empty());
    }

    #[test]
    fn ts_prose_passes() {
        let src = "// the socket closes after the last unsubscribe\nconst y = 1;\n";
        assert!(Language::TypeScript.source_rules(src).is_empty());
    }

    #[test]
    fn ts_allow_directive_works() {
        let src = "enum E {\n  // lint:allow(type-comment) legacy order is load-bearing\n  A,\n}\n";
        assert!(Language::TypeScript.source_rules(src).is_empty());
    }

    #[test]
    fn tsx_jsx_comment_passes_and_enum_fails() {
        let jsx = "export function App() {\n  return (\n    <div>\n      {/* layout wrapper */}\n      <span>hi</span>\n    </div>\n  );\n}\n";
        assert!(Language::Tsx.lint_source(jsx).is_empty(), "{:?}", Language::Tsx.lint_source(jsx));
        let enum_src = "enum E {\n  // note\n  A,\n}\n";
        assert_eq!(
            Language::Tsx
                .lint_source(enum_src)
                .into_iter()
                .map(|v| v.rule())
                .collect::<Vec<_>>(),
            vec!["type-comment"]
        );
    }
}
