use std::iter::successors;

use tree_sitter::Node;

use super::{RULE, Unit};
use crate::*;

/// A node carrying the source and grammar needed to score it.
#[derive(Clone, Copy)]
pub(super) struct Branch<'src> {
    node: Node<'src>,
    src: &'src str,
    language: Language,
}

impl<'src> Branch<'src> {
    pub(super) fn new(node: Node<'src>, src: &'src str, language: Language) -> Self {
        Self { node, src, language }
    }

    fn node(self) -> Node<'src> {
        self.node
    }

    fn src(self) -> &'src str {
        self.src
    }

    fn language(self) -> Language {
        self.language
    }

    pub(super) fn units(self) -> Vec<Unit> {
        self.unit()
            .into_iter()
            .chain(self.children().flat_map(Branch::units))
            .collect()
    }

    fn unit(self) -> Option<Unit> {
        self.scorable()
            .then(|| self.field("body"))
            .flatten()
            .map(Branch::branches)
            .map(|branches| Unit::new(self.describe(), self.node(), 1 + branches))
    }

    fn scorable(self) -> bool {
        self.is_unit() && !self.allows()
    }

    /// Sums the decision points under this node, stopping at a nested unit so
    /// each unit is counted once.
    fn branches(self) -> usize {
        self.scored()
            .map(Branch::branches)
            .sum::<usize>()
            .saturating_add(self.weight())
    }

    fn weight(self) -> usize {
        match (self.lookup_arm(), self.language()) {
            (true, _) => 0,
            (false, Language::Rust) => self.rust_weight(),
            (false, _) => self.ts_weight(),
        }
    }

    /// The first `match` arm is the fall-through, so only its alternatives
    /// branch. A let-chain joins its conditions outside `binary_expression`, so
    /// it counts its own conjuncts. `?` scores nothing: it is a return edge on
    /// every fallible call, and counting it would rank plumbing above branching.
    fn rust_weight(self) -> usize {
        match self.kind() {
            "if_expression" | "while_expression" | "loop_expression" | "for_expression" => 1,
            "match_arm" => usize::from(self.follows("match_arm")),
            "match_pattern" => usize::from(self.field("condition").is_some()),
            "let_chain" => self.node().named_child_count().saturating_sub(1),
            "let_declaration" => usize::from(self.field("alternative").is_some()),
            "binary_expression" => usize::from(matches!(self.operator(), "&&" | "||")),
            _ => 0,
        }
    }

    /// `switch_default` is the fall-through and carries its own node kind, so
    /// every `switch_case` counts.
    fn ts_weight(self) -> usize {
        match self.kind() {
            "if_statement" | "while_statement" | "do_statement" | "for_statement" | "for_in_statement"
            | "ternary_expression" | "catch_clause" | "switch_case" => 1,
            "binary_expression" => usize::from(matches!(self.operator(), "&&" | "||" | "??")),
            _ => 0,
        }
    }

    /// An arm that only produces a value is a lookup, not a branch. This is the
    /// one place the score departs from McCabe, and it is what keeps an
    /// exhaustive `match` over a domain enum from outscoring every tangled
    /// function we have.
    fn lookup_arm(self) -> bool {
        matches!(self.kind(), "match_arm" | "switch_case")
            && self.scored().map(Branch::branches).all(|score| score == 0)
    }

    fn describe(self) -> String {
        self.field("name")
            .or_else(|| self.binding())
            .map(Branch::text)
            .map(str::trim)
            .map_or_else(|| self.anonymous(), |name| format!("`{name}`"))
    }

    fn anonymous(self) -> String {
        self.ancestors()
            .filter_map(Branch::own_name)
            .map(Branch::text)
            .next()
            .map_or_else(|| "a closure".to_string(), |owner| format!("a closure in `{owner}`"))
    }

    fn own_name(self) -> Option<Self> {
        self.is_unit().then(|| self.field("name")).flatten()
    }

    /// A closure bound straight to a name is a function by another spelling.
    /// This covers `const Screen = () => ...`, the shape of a React component.
    fn binding(self) -> Option<Self> {
        self.parent().and_then(Branch::bound_name)
    }

    fn bound_name(self) -> Option<Self> {
        match self.kind() {
            "let_declaration" => self.field("pattern"),
            "const_item" | "static_item" | "variable_declarator" | "public_field_definition" => self.field("name"),
            "pair" => self.field("key"),
            "assignment_expression" => self.field("left"),
            _ => None,
        }
    }

    /// A unit opts out through a comment above it. Attributes and doc comments
    /// may sit between the directive and the signature.
    fn allows(self) -> bool {
        self.preceding()
            .take_while(Branch::is_trivia)
            .filter(Branch::is_comment)
            .map(Branch::text)
            .any(|text| Directive::grants(text, RULE))
    }

    fn children(self) -> impl Iterator<Item = Self> {
        (0..self.node().named_child_count())
            .filter_map(move |index| self.node().named_child(index))
            .map(move |node| self.at(node))
    }

    /// The children that carry this unit's own score. A nested unit is scored
    /// apart, so it contributes nothing here.
    fn scored(self) -> impl Iterator<Item = Self> {
        self.children().filter(|child| !child.is_unit())
    }

    fn ancestors(self) -> impl Iterator<Item = Self> {
        successors(self.node().parent(), Node::parent).map(move |node| self.at(node))
    }

    fn preceding(self) -> impl Iterator<Item = Self> {
        successors(self.node().prev_named_sibling(), Node::prev_named_sibling).map(move |node| self.at(node))
    }

    fn parent(self) -> Option<Self> {
        self.node().parent().map(|node| self.at(node))
    }

    fn field(self, name: &str) -> Option<Self> {
        self.node().child_by_field_name(name).map(|node| self.at(node))
    }

    /// Skips trivia on the way back. A trailing comment on the previous arm
    /// is a sibling between the two, and reading it as the predecessor scored
    /// every commented arm as the fall-through.
    fn follows(self, kind: &str) -> bool {
        self.preceding()
            .find(|previous| !previous.is_trivia())
            .is_some_and(|previous| previous.kind() == kind)
    }

    fn at(self, node: Node<'src>) -> Self {
        Self { node, ..self }
    }

    fn operator(self) -> &'src str {
        self.field("operator").map(Branch::text).unwrap_or_default()
    }

    fn kind(self) -> &'static str {
        self.node().kind()
    }

    fn text(self) -> &'src str {
        &self.src()[self.node().byte_range()]
    }

    fn is_unit(self) -> bool {
        self.language().unit_kinds().contains(&self.kind())
    }

    fn is_comment(&self) -> bool {
        matches!(self.kind(), "line_comment" | "block_comment" | "comment")
    }

    fn is_trivia(&self) -> bool {
        self.is_comment() || matches!(self.kind(), "attribute_item" | "decorator")
    }
}
