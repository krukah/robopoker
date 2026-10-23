use tree_sitter::Node;

use super::Kind;
use crate::*;

/// One declaration the shape rules judge, located for reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    kind: Kind,
    name: String,
    line: usize,
    end_line: usize,
    owner: Option<String>,
}

impl From<&Item> for Violation {
    fn from(item: &Item) -> Self {
        Self::new(item.line(), item.end_line(), item.kind().rule(), item.kind().message(item.name(), item.owner()))
    }
}

impl Item {
    pub(super) fn new(kind: Kind, name: String, node: Node<'_>) -> Self {
        Self {
            kind,
            name,
            line: node.top() + 1,
            end_line: node.bottom() + 1,
            owner: None,
        }
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn line(&self) -> usize {
        self.line
    }

    fn end_line(&self) -> usize {
        self.end_line
    }

    fn kind(&self) -> Kind {
        self.kind
    }

    fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    pub(super) fn is(&self, kind: Kind) -> bool {
        self.kind() == kind
    }

    /// Names the family that already owns this module, so the report says what
    /// this declaration lost to.
    pub(super) fn crowded_by(&self, owner: &str) -> Self {
        Self {
            owner: Some(owner.to_string()),
            ..self.clone()
        }
    }
}
