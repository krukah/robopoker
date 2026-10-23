use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::{Node, Parser};

use crate::*;

static RUST_CODE_KEYWORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:fn|let|pub|use|impl|struct|enum|trait|const|static|mod|match|if|else|for|while|loop|return|async|unsafe)\b")
        .unwrap()
});
static TS_CODE_KEYWORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:function|const|let|var|return|import|export|class|interface|type|enum|new|throw|await|async|switch|case|default|try|catch|if|else|for|while|do)\b")
        .unwrap()
});

const TS_CODE_NODE_KINDS: &[&str] = &[
    "call_expression",
    "assignment_expression",
    "augmented_assignment_expression",
    "lexical_declaration",
    "variable_declaration",
    "function_declaration",
    "class_declaration",
    "import_statement",
    "export_statement",
    "return_statement",
    "arrow_function",
    "new_expression",
    "interface_declaration",
    "type_alias_declaration",
    "enum_declaration",
    "throw_statement",
    "await_expression",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    Tsx,
}

impl Language {
    /// A `.d.ts` is a declaration file, not source, and its compound suffix
    /// reads as `ts` to `Path::extension`, so it is settled before the lookup.
    pub fn for_path(path: &str) -> Option<Self> {
        if path.ends_with(".d.ts") {
            return None;
        }
        match std::path::Path::new(path).extension()?.to_str()? {
            "rs" => Some(Self::Rust),
            "ts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            _ => None,
        }
    }

    pub(crate) fn parse(self, src: &str) -> Option<tree_sitter::Tree> {
        let mut parser = Parser::new();
        parser.set_language(&self.grammar()).expect("grammar loads");
        parser.parse(src, None)
    }

    fn grammar(self) -> tree_sitter::Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        }
    }

    pub(super) fn comment_shape(self, kind: &str, text: &str) -> Option<CommentShape> {
        match self {
            Self::Rust => match kind {
                "line_comment" => Some(CommentShape::Line),
                "block_comment" => Some(CommentShape::Block),
                _ => None,
            },
            Self::TypeScript | Self::Tsx => match kind {
                "comment" if text.starts_with("//") => Some(CommentShape::Line),
                "comment" => Some(CommentShape::Block),
                _ => None,
            },
        }
    }

    pub(super) fn is_doc_comment(self, text: &str) -> bool {
        let block_doc = text.starts_with("/**") && !text.starts_with("/***") && text != "/**/";
        match self {
            Self::Rust => {
                (text.starts_with("///") && !text.starts_with("////"))
                    || text.starts_with("//!")
                    || block_doc
                    || text.starts_with("/*!")
            }
            Self::TypeScript | Self::Tsx => block_doc,
        }
    }

    pub(super) fn banned_context(self, node: Node<'_>) -> Option<&'static str> {
        let parent = node.parent()?;
        let grandparent_kind = parent.parent().as_ref().map(Node::kind);
        match self {
            Self::Rust => match parent.kind() {
                "enum_variant_list" => Some("an enum body"),
                "field_declaration_list" => match grandparent_kind {
                    Some("union_item") => Some("a union body"),
                    Some("enum_variant") => Some("an enum body"),
                    _ => Some("a struct body"),
                },
                "ordered_field_declaration_list" => Some("a struct body"),
                "declaration_list" => match grandparent_kind {
                    Some("trait_item") => Some("a trait body"),
                    Some("impl_item") => Some("an impl body"),
                    _ => None,
                },
                "match_block" => Some("a match body"),
                _ => None,
            },
            Self::TypeScript | Self::Tsx => match parent.kind() {
                "enum_body" => Some("an enum body"),
                "interface_body" => Some("an interface body"),
                "object_type" => Some("a type-literal body"),
                "class_body" => Some("a class body"),
                "switch_body" => Some("a switch body"),
                _ => None,
            },
        }
    }

    pub(super) fn looks_like_code(self, body: &str) -> bool {
        let trimmed = body.trim();
        if trimmed.is_empty() {
            return false;
        }
        match self {
            Self::Rust => {
                let signal = trimmed.contains(';')
                    || trimmed.contains("::")
                    || trimmed.contains("=>")
                    || trimmed.contains('{')
                    || trimmed.contains('}')
                    || RUST_CODE_KEYWORD.is_match(trimmed);
                signal && (self.parses_clean(trimmed) || self.parses_clean(&format!("fn __lints() {{\n{trimmed}\n}}")))
            }
            Self::TypeScript | Self::Tsx => {
                let signal = trimmed.contains(';')
                    || trimmed.contains("=>")
                    || trimmed.contains('{')
                    || trimmed.contains('}')
                    || TS_CODE_KEYWORD.is_match(trimmed);
                signal
                    && self
                        .parse(trimmed)
                        .filter(|tree| !tree.root_node().has_error())
                        .is_some_and(|tree| Self::contains_kind(&tree, TS_CODE_NODE_KINDS))
            }
        }
    }

    fn parses_clean(self, src: &str) -> bool {
        self.parse(src).is_some_and(|tree| !tree.root_node().has_error())
    }

    fn contains_kind(tree: &tree_sitter::Tree, kinds: &[&str]) -> bool {
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if kinds.contains(&node.kind()) {
                return true;
            }
            for i in (0..node.child_count()).rev() {
                if let Some(child) = node.child(i) {
                    stack.push(child);
                }
            }
        }
        false
    }
}
