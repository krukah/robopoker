use super::*;
use crate::*;

pub const ONE_TYPE_PER_FILE: &str = "one-type-per-file";
pub const FREE_FUNCTION: &str = "free-function";
pub const PUB_FIELD: &str = "pub-field";
pub const FORWARDING_CLOSURE: &str = "forwarding-closure";
pub const TYPE_HINT: &str = "type-hint";
pub const INLINE_BOUND: &str = "inline-bound";
pub const FIELD_ACCESS: &str = "field-access";

impl Language {
    /// Judges every declaration in the file. A diff that reaches any line of a
    /// file answers for all of it.
    pub fn shape(self, path: &str, src: &str) -> Vec<Violation> {
        self.declarations(path, src).iter().map(Violation::from).collect()
    }

    fn declarations(self, path: &str, src: &str) -> Vec<Item> {
        let stem = Self::stem(path);
        match self {
            Self::Rust => self
                .parse(src)
                .map(|tree| Scan::new(tree.root_node(), src).report(stem))
                .unwrap_or_default(),
            Self::TypeScript | Self::Tsx => Vec::new(),
        }
    }

    fn stem(path: &str) -> &str {
        path.rsplit('/').next().unwrap_or_default().trim_end_matches(".rs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Language {
        fn shape_rules(self, src: &str) -> Vec<&'static str> {
            self.shape("crate/src/a.rs", src).iter().map(Violation::rule).collect()
        }

        fn count(self, src: &str, rule: &str) -> usize {
            self.shape_rules(src).iter().filter(|found| **found == rule).count()
        }
    }

    #[test]
    fn a_second_exposed_type_crowds_the_module() {
        assert_eq!(Language::Rust.count("pub struct A(u8);", ONE_TYPE_PER_FILE), 0);
        assert_eq!(Language::Rust.count("pub struct A(u8); pub enum B { X }", ONE_TYPE_PER_FILE), 1);
        assert_eq!(
            Language::Rust.count("pub struct A(u8); pub enum B { X } pub union C { a: u8 }", ONE_TYPE_PER_FILE),
            2
        );
    }

    #[test]
    fn the_file_s_namesake_owns_it() {
        let src = "pub enum OnCloseKind { A }\npub enum Tif { B }";
        let reported = Language::Rust.shape("order/src/tif.rs", src);
        assert_eq!(reported.len(), 1);
        assert!(
            reported[0]
                .message()
                .contains("`OnCloseKind` shares this module body with `Tif`")
        );
    }

    #[test]
    fn a_namesake_outranks_a_larger_family() {
        let src = "pub trait T { fn q(&self); }\npub struct A(u8);\nimpl T for A {}\npub struct Loner(u8);";
        assert_eq!(Language::Rust.shape("crate/src/loner.rs", src).len(), 2);
    }

    #[test]
    fn a_trait_still_gathers_its_implementors() {
        assert_eq!(Language::Rust.count("pub struct A(u8); pub trait Asks { fn q(&self); }", ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn a_type_declared_in_a_function_body_is_local_to_it() {
        let src = "pub struct A(u8);\nfn f() { pub struct Row(u8); }";
        assert_eq!(Language::Rust.count(src, ONE_TYPE_PER_FILE), 0);
    }

    #[test]
    fn an_inline_module_owns_its_own_budget() {
        let src = "pub struct A(u8);\npub mod m { pub struct B(u8); }";
        assert_eq!(Language::Rust.count(src, ONE_TYPE_PER_FILE), 0);
        let crowded = "pub mod m { pub struct B(u8); pub struct C(u8); }";
        assert_eq!(Language::Rust.count(crowded, ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn a_trait_gathers_the_types_that_implement_it() {
        let src =
            "pub trait T { fn q(&self); }\npub struct A(u8);\nimpl T for A {}\npub struct B(u8);\nimpl T for B {}";
        assert_eq!(Language::Rust.count(src, ONE_TYPE_PER_FILE), 0);
    }

    #[test]
    fn a_trait_gathers_only_its_own_implementors() {
        let src = "pub trait T { fn q(&self); }\npub struct A(u8);\nimpl T for A {}\npub struct Loner(u8);";
        assert_eq!(Language::Rust.shape_rules(src), vec![ONE_TYPE_PER_FILE]);
        let reported = Language::Rust.shape("crate/src/a.rs", src);
        assert!(
            reported[0]
                .message()
                .contains("`Loner` shares this module body with `T`")
        );
    }

    #[test]
    fn an_implementor_in_another_module_joins_no_family() {
        let src = "pub trait T { fn q(&self); }\npub struct A(u8);\npub mod m { impl super::T for super::A {} }";
        assert_eq!(Language::Rust.count(src, ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn a_type_alias_and_an_impl_are_not_types() {
        assert_eq!(
            Language::Rust.count("pub struct A(u8); pub type B = A; impl A { pub fn f(&self) {} }", ONE_TYPE_PER_FILE),
            0
        );
    }

    #[test]
    fn a_test_module_crowds_like_any_other() {
        let src = "pub struct A(u8);\n#[cfg(test)]\nmod tests { pub struct B(u8); pub struct C(u8); }";
        assert_eq!(Language::Rust.count(src, ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn a_private_type_crowds_all_the_same() {
        assert_eq!(Language::Rust.count("pub struct A(u8); struct B(u8);", ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn a_marker_crowds_all_the_same() {
        assert_eq!(Language::Rust.count("pub struct A(u8); pub struct B;", ONE_TYPE_PER_FILE), 1);
        assert_eq!(Language::Rust.count("pub struct A(u8); pub enum Never {}", ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn a_type_in_a_function_body_crowds_that_body() {
        let src = "fn f() { struct A(u8); struct B(u8); }";
        assert_eq!(Language::Rust.count(src, ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn a_directive_exempts_the_type_it_sits_above() {
        let src = "pub struct A(u8);\n// lint:allow(one-type-per-file) mirrors the vendor schema\npub struct B(u8);\npub struct C(u8);";
        assert_eq!(Language::Rust.count(src, ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn an_exempt_type_still_owns_the_module_it_opened() {
        let src = "// lint:allow(one-type-per-file) a reason\npub struct A(u8);\npub struct B(u8);";
        assert_eq!(Language::Rust.count(src, ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn a_forwarding_closure_names_its_method() {
        assert_eq!(Language::Rust.count("fn f() { g(|d| d.magnitude()); }", FORWARDING_CLOSURE), 1);
        assert_eq!(Language::Rust.count("fn f() { g(|d| d.scale(2)); }", FORWARDING_CLOSURE), 0);
        assert_eq!(Language::Rust.count("fn f() { g(|d| other.magnitude()); }", FORWARDING_CLOSURE), 0);
        assert_eq!(Language::Rust.count("fn f() { g(|d, e| d.magnitude()); }", FORWARDING_CLOSURE), 0);
        assert_eq!(Language::Rust.count("fn f() { g(|d| d.a().b()); }", FORWARDING_CLOSURE), 0);
        assert_eq!(Language::Rust.count("fn f() { g(|d, _| d.magnitude()); }", FORWARDING_CLOSURE), 0);
    }

    #[test]
    fn a_directive_above_a_chain_link_allows_its_closure() {
        let src = format!(
            "fn f() {{\n    x.iter()\n        // lint:allow({FORWARDING_CLOSURE}) depth\n        .filter(|d| d.magnitude());\n}}"
        );
        assert_eq!(Language::Rust.count(&src, FORWARDING_CLOSURE), 0);
        assert_eq!(Language::Rust.count(&src.replace("// lint", "// not"), FORWARDING_CLOSURE), 1);
    }

    #[test]
    fn a_field_reads_through_its_accessor() {
        assert_eq!(Language::Rust.count("impl A { fn f(&self) -> u8 { self.j + 1 } }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("impl A { fn f(&mut self) { self.j = 1; } }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("impl A { fn f(&self, o: &A) -> bool { o.j == 1 } }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("impl A { fn j(&self) -> u8 { self.j } }", FIELD_ACCESS), 0);
        assert_eq!(Language::Rust.count("impl A { fn j(&self) -> &[u8] { &self.j } }", FIELD_ACCESS), 0);
        assert_eq!(Language::Rust.count("impl A { fn j_mut(&mut self) -> &mut u8 { &mut self.j } }", FIELD_ACCESS), 0);
        assert_eq!(Language::Rust.count("impl A { fn j(&self) -> &[u8] { self.j.as_slice() } }", FIELD_ACCESS), 0);
        assert_eq!(Language::Rust.count("impl A { fn j(&self) -> u8 { self.j.k } }", FIELD_ACCESS), 2);
        assert_eq!(Language::Rust.count("impl A { fn j(&self, k: u8) -> u8 { self.j } }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("impl A { fn f(&self) -> u8 { self.j().k() } }", FIELD_ACCESS), 0);
        assert_eq!(Language::Rust.count("impl A { fn f(&self) { self.j.iter::<u8>(); } }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("impl A { fn f(&self) { (self.call)(1); } }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("impl A { fn f(&self) -> u8 { self.0 } }", FIELD_ACCESS), 0);
    }

    #[test]
    fn a_field_in_a_macro_reads_all_the_same() {
        assert_eq!(Language::Rust.count("fn main() { println!(\"{}\", self.j); }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("fn main() { println!(\"{}\", self.j()); }", FIELD_ACCESS), 0);
        assert_eq!(Language::Rust.count("fn main() { assert_eq!(a.b.c(), 1); }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("fn main() { vec![a::b(), c.d::<u8>()]; }", FIELD_ACCESS), 0);
        assert_eq!(Language::Rust.count("fn main() { println!(\"a.b {}\", 1.5); }", FIELD_ACCESS), 0);
    }

    #[test]
    fn a_test_reads_fields_through_accessors_too() {
        assert_eq!(Language::Rust.count("#[test]\nfn t() { assert!(a.j == 1); }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("#[cfg(test)]\nmod tests { fn t() { let _ = a.j; } }", FIELD_ACCESS), 1);
        assert_eq!(Language::Rust.count("#[cfg(test)]\nmod tests { #[test]\nfn t() { } }", FREE_FUNCTION), 0);
        assert_eq!(Language::Rust.count("#[cfg(test)]\nmod tests { fn helper() { } }", FREE_FUNCTION), 1);
        assert_eq!(Language::Rust.count("#[cfg(test)]\nmod tests { impl A { fn helper() { } } }", FREE_FUNCTION), 0);
    }

    #[test]
    fn a_function_with_no_owner_is_free() {
        assert_eq!(Language::Rust.count("fn helper() {}", FREE_FUNCTION), 1);
        assert_eq!(Language::Rust.count("mod m { fn helper() {} }", FREE_FUNCTION), 1);
        assert_eq!(Language::Rust.count("impl A { fn helper() {} }", FREE_FUNCTION), 0);
        assert_eq!(Language::Rust.count("trait A { fn helper(&self); }", FREE_FUNCTION), 0);
        assert_eq!(Language::Rust.count("fn main() {}", FREE_FUNCTION), 0);
        assert_eq!(Language::Rust.count("#[test]\nfn helper() {}", FREE_FUNCTION), 0);
        assert_eq!(Language::Rust.count("#[component]\nfn Seat() {}", FREE_FUNCTION), 0);
        assert_eq!(Language::Rust.count("#[proc_macro_derive(A)]\nfn d() {}", FREE_FUNCTION), 0);
        assert_eq!(Language::Rust.count("#[inline]\nfn helper() {}", FREE_FUNCTION), 1);
    }

    #[test]
    fn cfg_alternatives_of_one_name_are_one_type() {
        let src = "#[cfg(a)]\npub enum R { X }\n#[cfg(not(a))]\npub enum R { Y }";
        assert_eq!(Language::Rust.count(src, ONE_TYPE_PER_FILE), 0);
        assert_eq!(Language::Rust.count("pub struct A;\npub struct B;", ONE_TYPE_PER_FILE), 1);
    }

    #[test]
    fn a_dto_derive_earns_exposed_fields() {
        assert_eq!(Language::Rust.count("struct A { pub a: u8, b: u8 }", PUB_FIELD), 1);
        assert_eq!(Language::Rust.count("#[derive(Deserialize)]\nstruct A { pub a: u8 }", PUB_FIELD), 0);
        assert_eq!(Language::Rust.count("#[derive(Debug, sqlx::FromRow)]\nstruct A { pub a: u8 }", PUB_FIELD), 0);
        assert_eq!(Language::Rust.count("#[derive(Parser)]\nstruct A { pub a: u8 }", PUB_FIELD), 0);
        assert_eq!(Language::Rust.count("#[derive(Clone, FromRef)]\nstruct A { pub a: u8 }", PUB_FIELD), 0);
    }

    #[test]
    fn the_query_row_marker_earns_exposed_fields() {
        assert_eq!(Language::Rust.count("struct A { pub a: u8 }", PUB_FIELD), 1);
        assert_eq!(Language::Rust.count("#[derive(macros::QueryRow)]\nstruct A { pub a: u8 }", PUB_FIELD), 0);
        assert_eq!(
            Language::Rust
                .count("#[derive(Debug, QueryRow)]\nstruct A { pub(crate) a: u8, pub(super) b: u8 }", PUB_FIELD),
            0
        );
    }

    #[test]
    fn every_visibility_exposes_a_field() {
        assert_eq!(Language::Rust.count("struct A { pub(crate) a: u8 }", PUB_FIELD), 1);
        assert_eq!(Language::Rust.count("struct A { pub(super) a: u8 }", PUB_FIELD), 1);
        assert_eq!(Language::Rust.count("struct A { pub(in crate::m) a: u8 }", PUB_FIELD), 1);
        assert_eq!(Language::Rust.count("struct A { a: u8 }", PUB_FIELD), 0);
    }

    #[test]
    fn a_tuple_field_answers_to_no_rule() {
        assert_eq!(Language::Rust.count("pub struct A(pub u8);", PUB_FIELD), 0);
    }

    #[test]
    fn a_directive_over_the_struct_exempts_every_field() {
        let src = "// lint:allow(pub-field) the vendor fills it field by field\npub struct A { pub a: u8, pub b: u8 }";
        assert_eq!(Language::Rust.count(src, PUB_FIELD), 0);
        let one = "pub struct A { // lint:allow(pub-field) a reason\n pub a: u8, pub b: u8 }";
        assert_eq!(Language::Rust.count(one, PUB_FIELD), 1);
    }

    #[test]
    fn a_type_hint_only_counts_where_a_turbofish_fits() {
        assert_eq!(Language::Rust.count("fn f() { let a: Vec<u8> = b.collect(); }", TYPE_HINT), 1);
        assert_eq!(Language::Rust.count("fn f() { let a: Vec<u8> = b.collect::<Vec<u8>>(); }", TYPE_HINT), 0);
        assert_eq!(Language::Rust.count("fn f() { let a: Vec<u8> = b.clone(); }", TYPE_HINT), 0);
        assert_eq!(Language::Rust.count("fn f() { let a: u8 = 1; }", TYPE_HINT), 0);
    }

    #[test]
    fn an_inline_bound_counts_but_a_lifetime_does_not() {
        assert_eq!(Language::Rust.count("fn f<T: Clone>(a: T) {}", INLINE_BOUND), 1);
        assert_eq!(Language::Rust.count("fn f<T>(a: T) where T: Clone {}", INLINE_BOUND), 0);
        assert_eq!(Language::Rust.count("fn f<'outer: 'inner, 'inner>(a: &'outer u8) {}", INLINE_BOUND), 0);
        assert_eq!(Language::Rust.count("trait A: B {}", INLINE_BOUND), 0);
    }

    #[test]
    fn a_test_file_answers_like_any_other() {
        let src = "pub struct A(u8); pub struct B(u8);";
        assert_eq!(Language::Rust.shape("crate/src/a_intg_test.rs", src).len(), 1);
        assert_eq!(Language::Rust.shape("crate/tests/a.rs", src).len(), 1);
    }

    #[test]
    fn typescript_answers_to_nothing() {
        let src = "interface A { a: number } interface B { b: number }";
        assert!(Language::TypeScript.shape("a.ts", src).is_empty());
    }

    #[test]
    fn a_touched_file_answers_for_every_declaration() {
        let src = "pub struct A(u8);\npub struct B(u8);\npub struct C(u8);\n";
        assert_eq!(Language::Rust.shape("a.rs", src).len(), 2);
    }

    #[test]
    fn a_directive_above_a_declaration_exempts_it() {
        let src = "// lint:allow(free-function) the binary entry point\nfn helper() {}";
        assert_eq!(Language::Rust.count(src, FREE_FUNCTION), 0);
        let attributed = "// lint:allow(free-function) a reason\n#[inline]\nfn helper() {}";
        assert_eq!(Language::Rust.count(attributed, FREE_FUNCTION), 0);
    }
}
