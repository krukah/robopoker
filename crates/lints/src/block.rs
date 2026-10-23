use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::Node;

use crate::*;

static CFG_TEST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"cfg\s*\([^)]*\btest\b").unwrap());
static TEST_ATTR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#\[\s*(?:[\w]+\s*::\s*)*(?:test|rstest|bench|intg_test)\b").unwrap());

#[derive(Debug)]
pub(crate) struct Block {
    start_line: usize,
    end_line: usize,
    content: Vec<String>,
    context: Option<&'static str>,
    mergeable: bool,
    own_line: bool,
    doc: bool,
    hidden: Option<(&'static str, usize)>,
    member_doc: bool,
}

impl Block {
    pub(crate) fn collect(src: &str, language: Language) -> Vec<Self> {
        let Some(tree) = language.parse(src) else {
            return Vec::new();
        };
        let mut blocks: Vec<Block> = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            for i in (0..node.child_count()).rev() {
                if let Some(child) = node.child(i) {
                    stack.push(child);
                }
            }
            let Some(shape) = language.comment_shape(node.kind(), &src[node.byte_range()]) else {
                continue;
            };
            let text = src[node.byte_range()].trim_end_matches(['\n', '\r']);
            let doc = language.is_doc_comment(text);
            let start_line = node.top() + 1;
            let end_line = start_line + text.matches('\n').count();
            let own_line = Self::starts_own_line(src, node.start_byte());
            let content = match shape {
                CommentShape::Block => Self::block_comment_lines(text, doc),
                CommentShape::Line => {
                    let stripped = text.strip_prefix("//").unwrap_or(text);
                    let stripped = if doc {
                        stripped
                            .strip_prefix('/')
                            .or_else(|| stripped.strip_prefix('!'))
                            .unwrap_or(stripped)
                    } else {
                        stripped
                    };
                    vec![stripped.to_string()]
                }
            };
            let context = language.banned_context(node);
            let mergeable = shape == CommentShape::Line && own_line;
            let inner_doc = text.starts_with("//!") || text.starts_with("/*!");
            let hidden = match (language, doc) {
                (Language::Rust, true) => Self::hidden_reason(node, src, inner_doc),
                _ => None,
            };
            let member_doc = doc && !inner_doc && Self::documents_member(node, language);
            let block = Self {
                start_line,
                end_line,
                content,
                context,
                mergeable,
                own_line,
                doc,
                hidden,
                member_doc,
            };
            match blocks.pop() {
                Some(last) if last.joins(&block) => blocks.push(last.merge(&block)),
                Some(last) => blocks.extend([last, block]),
                None => blocks.push(block),
            }
        }
        blocks
    }

    fn joins(&self, next: &Self) -> bool {
        self.mergeable() && next.mergeable() && self.doc() == next.doc() && self.end_line() + 1 == next.start_line()
    }

    fn merge(self, next: &Self) -> Self {
        Self {
            end_line: next.end_line(),
            content: self.content().iter().chain(next.content()).cloned().collect(),
            context: self.context().or(next.context()),
            hidden: self.hidden().or(next.hidden()),
            member_doc: self.member_doc() || next.member_doc(),
            ..self
        }
    }

    fn hidden_reason(node: Node<'_>, src: &str, inner_doc: bool) -> Option<(&'static str, usize)> {
        if Self::in_test_scope(node, src) {
            return Some(("a `#[cfg(test)]` item", MAX_COMMENT_LINES));
        }
        if inner_doc {
            return None;
        }
        let (item, test_attr) = Self::documented_item(node, src);
        if test_attr {
            return Some(("a test item", MAX_COMMENT_LINES));
        }
        let item = item?;
        if Self::has_visibility(item) {
            return None;
        }
        match item.kind() {
            "enum_variant" => None,
            "function_item" | "const_item" | "type_item" if Self::in_trait_surface(item) => None,
            "field_declaration" => Some(("a private field", MAX_COMMENT_LINES)),
            "function_item" => Some(("a private function", MAX_DOC_COMMENT_LINES)),
            "const_item" | "static_item" => Some(("a private constant", MAX_DOC_COMMENT_LINES)),
            "struct_item" | "enum_item" | "union_item" | "type_item" | "trait_item" | "mod_item" => {
                Some(("a private item", MAX_DOC_COMMENT_LINES))
            }
            _ => None,
        }
    }

    fn documents_member(node: Node<'_>, language: Language) -> bool {
        let Some(item) = Self::next_item(node) else {
            return false;
        };
        match language {
            Language::Rust => matches!(item.kind(), "enum_variant" | "field_declaration"),
            Language::TypeScript | Language::Tsx => {
                matches!(item.kind(), "property_signature" | "public_field_definition" | "enum_assignment")
            }
        }
    }

    fn next_item(node: Node<'_>) -> Option<Node<'_>> {
        let mut sibling = node.next_named_sibling();
        while let Some(next) = sibling {
            match next.kind() {
                "line_comment" | "block_comment" | "comment" | "attribute_item" => {}
                _ => return Some(next),
            }
            sibling = next.next_named_sibling();
        }
        None
    }

    fn in_test_scope(node: Node<'_>, src: &str) -> bool {
        let mut current = Some(node);
        while let Some(scope) = current {
            let mut sibling = scope.prev_named_sibling();
            while let Some(previous) = sibling {
                match previous.kind() {
                    "attribute_item" if Self::is_test_attribute(&src[previous.byte_range()]) => return true,
                    "attribute_item" | "line_comment" | "block_comment" => {}
                    _ => break,
                }
                sibling = previous.prev_named_sibling();
            }
            current = scope.parent();
        }
        false
    }

    fn documented_item<'tree>(node: Node<'tree>, src: &str) -> (Option<Node<'tree>>, bool) {
        let mut sibling = node.next_named_sibling();
        let mut test_attr = false;
        while let Some(next) = sibling {
            match next.kind() {
                "line_comment" | "block_comment" => {}
                "attribute_item" => test_attr |= Self::is_test_attribute(&src[next.byte_range()]),
                _ => return (Some(next), test_attr),
            }
            sibling = next.next_named_sibling();
        }
        (None, test_attr)
    }

    pub(crate) fn is_test_attribute(text: &str) -> bool {
        CFG_TEST.is_match(text) || TEST_ATTR.is_match(text)
    }

    fn has_visibility(item: Node<'_>) -> bool {
        (0..item.child_count())
            .filter_map(|i| item.child(i))
            .any(|child| child.kind() == "visibility_modifier")
    }

    fn in_trait_surface(item: Node<'_>) -> bool {
        let Some(owner) = item.parent().as_ref().and_then(Node::parent) else {
            return false;
        };
        match owner.kind() {
            "trait_item" => true,
            "impl_item" => owner.child_by_field_name("trait").is_some(),
            _ => false,
        }
    }

    fn starts_own_line(src: &str, start_byte: usize) -> bool {
        src.as_bytes()[..start_byte]
            .iter()
            .rev()
            .take_while(|&&b| b != b'\n')
            .all(u8::is_ascii_whitespace)
    }

    fn block_comment_lines(text: &str, doc: bool) -> Vec<String> {
        let inner = text.strip_prefix("/*").unwrap_or(text);
        let inner = inner.strip_suffix("*/").unwrap_or(inner);
        inner
            .split('\n')
            .map(|line| {
                let line = line.trim_start();
                let line = line.strip_prefix('*').unwrap_or(line);
                let line = if doc { line.strip_prefix('!').unwrap_or(line) } else { line };
                line.to_string()
            })
            .collect()
    }

    pub(crate) fn allows_rule(&self, rule: &str) -> bool {
        Directive::parse(self).is_some_and(|directive| directive.allows().contains(rule))
    }

    pub(crate) fn start_line(&self) -> usize {
        self.start_line
    }

    pub(crate) fn end_line(&self) -> usize {
        self.end_line
    }

    pub(crate) fn content(&self) -> &[String] {
        &self.content
    }

    pub(crate) fn context(&self) -> Option<&'static str> {
        self.context
    }

    pub(crate) fn mergeable(&self) -> bool {
        self.mergeable
    }

    pub(crate) fn own_line(&self) -> bool {
        self.own_line
    }

    pub(crate) fn doc(&self) -> bool {
        self.doc
    }

    pub(crate) fn hidden(&self) -> Option<(&'static str, usize)> {
        self.hidden
    }

    pub(crate) fn member_doc(&self) -> bool {
        self.member_doc
    }
}
