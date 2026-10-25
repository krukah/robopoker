use tree_sitter::Node;

use crate::*;

pub const WIRING_FILE: &str = "wiring-file";
pub const ENTRY_FILE: &str = "entry-file";

/// A file that Cargo or a module tree names, and so may hold no logic of its
/// own. `lib.rs` and `mod.rs` wire modules together; `main.rs` wires them and
/// starts the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    Wiring,
    Main,
}

/// What may sit between an attribute and the item it marks.
const TRIVIA: &[&str] = &["attribute_item", "line_comment", "block_comment"];

/// Every item that only wires a module tree together.
const WIRING: &[&str] = &[
    "use_declaration",
    "extern_crate_declaration",
    "attribute_item",
    "inner_attribute_item",
    "line_comment",
    "block_comment",
    "shebang",
];

impl Entry {
    pub fn for_path(path: &str) -> Option<Self> {
        match path.rsplit('/').next().unwrap_or_default() {
            "lib.rs" | "mod.rs" => Some(Self::Wiring),
            "main.rs" => Some(Self::Main),
            _ => None,
        }
    }

    /// Reports every item in the file that is neither wiring nor, in a
    /// `main.rs`, `fn main` itself.
    pub fn check(path: &str, src: &str) -> Vec<Violation> {
        let Some(entry) = Self::for_path(path) else {
            return Vec::new();
        };
        if entry.exempt(src) {
            return Vec::new();
        }
        let Some(tree) = Language::Rust.parse(src) else {
            return Vec::new();
        };
        let root = tree.root_node();
        (0..root.named_child_count())
            .filter_map(|index| root.named_child(index))
            .filter(|node| !entry.wires(*node, src))
            .filter(|node| !Self::allowed(src, node.top(), entry.rule()))
            .map(|node| entry.violation(node, src))
            .collect()
    }

    fn rule(self) -> &'static str {
        match self {
            Self::Wiring => WIRING_FILE,
            Self::Main => ENTRY_FILE,
        }
    }

    fn wires(self, node: Node<'_>, src: &str) -> bool {
        if WIRING.contains(&node.kind()) {
            return true;
        }
        if node.kind() == "mod_item" {
            return node.child_by_field_name("body").is_none();
        }
        if node.kind() == "function_item" && Self::proc_macro(node, src) {
            return true;
        }
        self == Self::Main
            && node.kind() == "function_item"
            && node
                .child_by_field_name("name")
                .is_some_and(|child| Self::text(child, src) == "main")
    }

    fn violation(self, node: Node<'_>, src: &str) -> Violation {
        let noun = Self::noun(node.kind());
        let name = Self::name(node, src);
        let message = match self {
            Self::Wiring => format!(
                "`{name}` is a {noun} in a wiring file. A `lib.rs` or `mod.rs` declares modules and re-exports them, nothing else. Move the {noun} to its own file"
            ),
            Self::Main => format!(
                "`{name}` is a {noun} outside `fn main`. A `main.rs` holds the entry point and its imports, nothing else. Move the {noun} to its own file"
            ),
        };
        Violation::new(node.top() + 1, node.bottom() + 1, self.rule(), message)
    }

    /// A directive on the first line of the file exempts the whole file.
    fn exempt(self, src: &str) -> bool {
        Directive::heads(src, self.rule())
    }

    /// A procedural macro must be declared in the crate root. Rust puts it there,
    /// not the author. A comment may sit between the attribute and the function,
    /// so the walk reads through trivia and stops at the first real item.
    fn proc_macro(node: Node<'_>, src: &str) -> bool {
        std::iter::successors(node.prev_named_sibling(), Node::prev_named_sibling)
            .take_while(|prior| TRIVIA.contains(&prior.kind()))
            .filter(|prior| prior.kind() == "attribute_item")
            .any(|prior| Self::marks_a_proc_macro(Self::text(prior, src)))
    }

    /// Reads the attribute's own path, so `#[doc = "proc_macro helper"]` marks
    /// nothing.
    fn marks_a_proc_macro(attribute: &str) -> bool {
        let path = attribute
            .trim_start_matches("#[")
            .trim_start()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
            .unwrap_or_default();
        path == "proc_macro" || path.starts_with("proc_macro_")
    }

    /// A directive in the comment block directly above the item exempts it.
    fn allowed(src: &str, row: usize, rule: &str) -> bool {
        src.lines()
            .take(row)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .take_while(|line| {
                let line = line.trim_start();
                line.starts_with("//") || line.starts_with("#[")
            })
            .any(|line| Directive::grants(line.trim(), rule))
    }

    fn name(node: Node<'_>, src: &str) -> String {
        for field in ["name", "type", "pattern"] {
            if let Some(child) = node.child_by_field_name(field) {
                return Self::text(child, src).trim().to_string();
            }
        }
        Self::text(node, src)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    fn text<'src>(node: Node<'_>, src: &'src str) -> &'src str {
        src.get(node.byte_range()).unwrap_or_default()
    }

    fn noun(kind: &str) -> &'static str {
        match kind {
            "struct_item" => "struct",
            "enum_item" => "enum",
            "union_item" => "union",
            "trait_item" => "trait",
            "impl_item" => "impl block",
            "function_item" | "function_signature_item" => "function",
            "const_item" => "const",
            "static_item" => "static",
            "type_item" => "type alias",
            "macro_definition" => "macro",
            "macro_invocation" | "expression_statement" => "macro invocation",
            "mod_item" => "inline module",
            "foreign_mod_item" => "extern block",
            "associated_type" => "associated type",
            _ => "declaration",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Entry {
        fn rules(path: &str, src: &str) -> Vec<&'static str> {
            Self::check(path, src).iter().map(Violation::rule).collect()
        }
    }

    #[test]
    fn a_wiring_file_takes_modules_and_re_exports() {
        let src =
            "#![allow(dead_code)]\n// a note\nmod a;\npub mod b;\npub use a::*;\nuse std::fmt;\nextern crate alloc;\n";
        assert!(Entry::rules("crate/src/lib.rs", src).is_empty());
        assert!(Entry::rules("crate/src/a/mod.rs", src).is_empty());
    }

    #[test]
    fn a_wiring_file_takes_no_declaration() {
        assert_eq!(Entry::rules("c/src/lib.rs", "pub struct A(u8);"), vec![WIRING_FILE]);
        assert_eq!(Entry::rules("c/src/mod.rs", "const N: u8 = 1;"), vec![WIRING_FILE]);
        assert_eq!(Entry::rules("c/src/lib.rs", "fn f() {}"), vec![WIRING_FILE]);
        assert_eq!(Entry::rules("c/src/lib.rs", "impl A { fn f(&self) {} }"), vec![WIRING_FILE]);
        assert_eq!(Entry::rules("c/src/lib.rs", "thread_local! { static A: u8 = 1; }"), vec![WIRING_FILE]);
    }

    #[test]
    fn an_inline_module_is_a_body_a_wiring_file_may_not_hold() {
        assert_eq!(Entry::rules("c/src/lib.rs", "mod a { pub struct A(u8); }"), vec![WIRING_FILE]);
        assert_eq!(Entry::rules("c/src/lib.rs", "#[cfg(test)]\nmod tests { #[test] fn t() {} }"), vec![WIRING_FILE]);
    }

    #[test]
    fn a_main_file_keeps_its_entry_point_and_nothing_else() {
        assert!(Entry::rules("c/src/main.rs", "use a::b;\nmod c;\nfn main() {}").is_empty());
        assert_eq!(Entry::rules("c/src/main.rs", "fn main() {}\nfn helper() {}"), vec![ENTRY_FILE]);
        assert_eq!(Entry::rules("c/src/main.rs", "struct Args;\nfn main() {}"), vec![ENTRY_FILE]);
    }

    #[test]
    fn a_body_inside_fn_main_is_the_point() {
        let src = "fn main() {\n    struct Local(u8);\n    let a = Local(1);\n}";
        assert!(Entry::rules("c/src/main.rs", src).is_empty());
    }

    #[test]
    fn every_other_file_answers_to_neither_rule() {
        assert!(Entry::rules("c/src/order.rs", "pub struct A(u8);\nfn f() {}").is_empty());
        assert!(Entry::rules("c/src/lib.ts", "export const a = 1;").is_empty());
    }

    #[test]
    fn rust_puts_a_procedural_macro_in_the_crate_root() {
        let src = "#[proc_macro_derive(QueryRow)]\npub fn derive(input: TokenStream) -> TokenStream { input }";
        assert!(Entry::rules("macros/src/lib.rs", src).is_empty());
        let plain = "pub fn derive(input: TokenStream) -> TokenStream { input }";
        assert_eq!(Entry::rules("macros/src/lib.rs", plain), vec![WIRING_FILE]);
    }

    #[test]
    fn a_comment_does_not_break_the_proc_macro_exception() {
        let src = "#[proc_macro_attribute]\n// what it expands to\npub fn mark(a: TokenStream, b: TokenStream) -> TokenStream { b }";
        assert!(Entry::rules("macros/src/lib.rs", src).is_empty());
    }

    #[test]
    fn only_the_attribute_path_marks_a_proc_macro() {
        let src = "#[doc = \"the proc_macro helper\"]\npub fn helper() {}";
        assert_eq!(Entry::rules("macros/src/lib.rs", src), vec![WIRING_FILE]);
    }

    #[test]
    fn a_directive_above_the_item_exempts_it() {
        let src = "mod a;\n// lint:allow(wiring-file) the macro writes the module tree\ntonic::include_proto!(\"v1\");\npub struct A(u8);";
        assert_eq!(Entry::rules("c/src/lib.rs", src), vec![WIRING_FILE]);
        let attributed = "mod a;\n// lint:allow(entry-file) a reason\n#[derive(Debug)]\nstruct Args;\nfn main() {}";
        assert!(Entry::rules("c/src/main.rs", attributed).is_empty());
    }

    #[test]
    fn a_directive_on_the_first_line_exempts_the_file() {
        let src = "// lint:allow(wiring-file) generated bindings land here\npub struct A(u8);\npub struct B(u8);";
        assert!(Entry::rules("c/src/lib.rs", src).is_empty());
    }
}
