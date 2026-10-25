use std::ops::Range;

use tree_sitter::Node;

use super::{MAX_COMPLEXITY, RULE};
use crate::*;

/// One function, method, or closure. A nested unit scores on its own, so an
/// iterator chain is not charged to the function that holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    name: String,
    line: usize,
    end_line: usize,
    score: usize,
}

impl From<Unit> for Violation {
    fn from(unit: Unit) -> Self {
        Self::new(
            unit.line(),
            unit.line(),
            RULE,
            format!(
                "{} has a branch complexity of {}; the maximum is {MAX_COMPLEXITY}. Lift a branch into a named helper, or turn the condition into a type the compiler dispatches on",
                unit.name(),
                unit.score()
            ),
        )
    }
}

impl Unit {
    pub(super) fn new(name: String, node: Node<'_>, score: usize) -> Self {
        Self {
            name,
            line: node.top() + 1,
            end_line: node.bottom() + 1,
            score,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn score(&self) -> usize {
        self.score
    }

    pub(super) fn by_line(mut units: Vec<Self>) -> Vec<Self> {
        units.sort_by_key(Self::line);
        units
    }

    pub(super) fn over_limit(&self) -> bool {
        self.score() > MAX_COMPLEXITY
    }

    /// A diff that reaches no line of this unit leaves it alone. Legacy code
    /// stays silent until someone edits it.
    pub(super) fn touched(&self, changed: Option<&[Range<usize>]>) -> bool {
        changed.is_none_or(|ranges| ranges.iter().any(|range| self.overlaps(range)))
    }

    fn overlaps(&self, Range { start, end }: &Range<usize>) -> bool {
        *start <= self.end_line() && self.line() < *end
    }

    fn end_line(&self) -> usize {
        self.end_line
    }
}
