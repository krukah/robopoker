use super::*;
use deuce::*;

/// Result of replaying one hand.
#[derive(Debug, Clone)]
pub struct Outcome {
    finishing: Vec<Halves>,
    nodes: Vec<Node>,
    board: Vec<Card>,
    flopped: Vec<usize>,
    showdown: bool,
    odd_chips: bool,
}
impl Outcome {
    pub fn new(
        finishing: Vec<Halves>,
        nodes: Vec<Node>,
        board: Vec<Card>,
        flopped: Vec<usize>,
        showdown: bool,
        odd_chips: bool,
    ) -> Self {
        Self {
            finishing,
            nodes,
            board,
            flopped,
            showdown,
            odd_chips,
        }
    }

    /// Per-seat stacks after settlement, in half-chips.
    pub fn finishing(&self) -> &[Halves] {
        &self.finishing
    }

    /// Decision nodes in the order they occurred.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn board(&self) -> &[Card] {
        &self.board
    }

    /// Seats still in the hand when the flop was dealt. Empty if the hand
    /// ended preflop.
    pub fn flopped(&self) -> &[usize] {
        &self.flopped
    }

    /// True when two or more players were live at the end of the river.
    pub fn showdown(&self) -> bool {
        self.showdown
    }

    /// True when a pot could not be split evenly even in half-chips, so the
    /// remainder was awarded to the earliest seat.
    pub fn odd_chips(&self) -> bool {
        self.odd_chips
    }

    /// Whether the replay reproduces the logged `finishing_stacks` exactly.
    pub fn agrees_with(&self, record: &Record) -> bool {
        let logged = record.finishing_stacks();
        logged.len() == self.finishing().len() && logged.iter().zip(self.finishing().iter()).all(|(l, f)| l == f)
    }
}

impl Outcome {}
