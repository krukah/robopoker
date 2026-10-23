use tree_sitter::Node;

/// The body that holds a declaration, identified by its node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Scope(usize);

impl Scope {
    /// The module body a declaration enters: a file, or a `mod` block.
    pub(super) fn of(node: Node<'_>) -> Option<Self> {
        let parent = node.parent()?;
        match parent.kind() {
            "source_file" => Some(Self(parent.id())),
            "declaration_list" => Self::under_a_module(parent),
            _ => None,
        }
    }

    /// The body a declaration sits in, module or not. A type declared inside
    /// a function crowds that function's body the same way it would a module.
    pub(super) fn holding(node: Node<'_>) -> Option<Self> {
        node.parent().map(|parent| Self(parent.id()))
    }

    fn under_a_module(body: Node<'_>) -> Option<Self> {
        body.parent()
            .filter(|owner| owner.kind() == "mod_item")
            .map(|_| Self(body.id()))
    }
}
