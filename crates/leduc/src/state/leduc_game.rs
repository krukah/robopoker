use super::node::Node;
use crate::*;
use mccfr::*;
use pokerkit::*;

/// Game state for Leduc Hold'em.
///
/// Hole cards are dealt at root. The `Node` enum encodes the
/// full game phase with zero invalid states.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeducGame {
    hole: [Card; 2],
    node: Node,
}
impl LeducGame {
    fn hole(&self) -> [Card; 2] {
        self.hole
    }

    fn hole_mut(&mut self) -> &mut [Card; 2] {
        &mut self.hole
    }

    fn node(&self) -> Node {
        self.node
    }

    pub(crate) fn spots(&self) -> (Spot, Option<Spot>) {
        match &self.node() {
            Node::Start | Node::Dealt => (Spot::Open, None),
            Node::R1(s) => (*s, None),
            Node::Deal(s) => (*s, Some(Spot::Open)),
            Node::R2(_, r1, r2) => (*r1, Some(*r2)),
            Node::Over(Outcome::FoldR1(_)) => (Spot::Open, None),
            Node::Over(Outcome::FoldR2(_, r1, _)) => (*r1, None),
            Node::Over(Outcome::Showdown(_, r1, r2)) => (*r1, Some(*r2)),
        }
    }

    pub fn with_card(mut self, actor: usize, card: Card) -> Self {
        self.hole_mut()[actor] = card;
        self
    }

    pub fn hole_card(&self, actor: usize) -> Card {
        self.hole()[actor]
    }

    pub fn board(&self) -> Option<Card> {
        match &self.node() {
            Node::R2(c, _, _) => Some(*c),
            Node::Over(Outcome::FoldR2(c, _, _)) => Some(*c),
            Node::Over(Outcome::Showdown(c, _, _)) => Some(*c),
            _ => None,
        }
    }

    pub fn board_rank(&self) -> Option<Rank> {
        self.board().map(crate::Card::rank)
    }

    pub fn hole_rank(&self, actor: usize) -> Rank {
        self.hole()[actor].rank()
    }

    pub fn deals(&self) -> impl Iterator<Item = Card> + '_ {
        let h0 = !matches!(self.node(), Node::Start);
        let h1 = !matches!(self.node(), Node::Start | Node::Dealt);
        let board = self.board();
        Card::ALL
            .into_iter()
            .filter(move |c| !h0 || *c != self.hole()[0])
            .filter(move |c| !h1 || *c != self.hole()[1])
            .filter(move |c| board != Some(*c))
    }

    pub fn all_roots() -> impl Iterator<Item = Self> {
        Card::ALL.into_iter().flat_map(|c0| {
            Card::ALL.into_iter().filter(move |c1| *c1 != c0).map(move |c1| Self {
                hole: [c0, c1],
                node: Node::R1(Spot::Open),
            })
        })
    }
}
impl CfrGame for LeducGame {
    type E = LeducEdge;
    type T = LeducTurn;

    fn root() -> Self {
        let mut cards = Card::ALL;
        cards.swap(0, rand::random_range(0..6));
        cards.swap(1, rand::random_range(1..6));
        Self {
            hole: [cards[0], cards[1]],
            node: Node::R1(Spot::Open),
        }
    }

    fn turn(&self) -> Self::T {
        match &self.node() {
            Node::Start | Node::Dealt | Node::Deal(_) => LeducTurn::Chance,
            Node::Over(_) => LeducTurn::Terminal,
            Node::R1(spot) | Node::R2(_, _, spot) => LeducTurn::Player(spot.actor()),
        }
    }
    #[rustfmt::skip]
    fn apply(&self, edge: Self::E) -> Self {
        if let (Node::Start, LeducEdge::Deal(c)) = (&self.node(), edge) {
            return Self { hole: [c, self.hole()[1]], node: Node::Dealt };
        }
        if let (Node::Dealt, LeducEdge::Deal(c)) = (&self.node(), edge) {
            return Self { hole: [self.hole()[0], c], node: Node::R1(Spot::Open) };
        }
        let node = match (&self.node(), edge) {
            (Node::R1(Spot::Open), LeducEdge::Check)              => Node::R1(Spot::Checked),
            (Node::R1(Spot::Open), LeducEdge::Raise)              => Node::R1(Spot::Raised),
            (Node::R1(Spot::Checked), LeducEdge::Check)           => Node::Deal(Spot::Checked),
            (Node::R1(Spot::Checked), LeducEdge::Raise)           => Node::R1(Spot::CheckRaised),
            (Node::R1(Spot::Raised), LeducEdge::Call)             => Node::Deal(Spot::Raised),
            (Node::R1(Spot::Raised), LeducEdge::Fold)             => Node::Over(Outcome::FoldR1(1)),
            (Node::R1(Spot::CheckRaised), LeducEdge::Call)        => Node::Deal(Spot::CheckRaised),
            (Node::R1(Spot::CheckRaised), LeducEdge::Fold)        => Node::Over(Outcome::FoldR1(0)),
            (Node::Deal(r), LeducEdge::Deal(c))                   => Node::R2(c, *r, Spot::Open),
            (Node::R2(c, r, Spot::Open), LeducEdge::Check)        => Node::R2(*c, *r, Spot::Checked),
            (Node::R2(c, r, Spot::Open), LeducEdge::Raise)        => Node::R2(*c, *r, Spot::Raised),
            (Node::R2(c, r, Spot::Checked), LeducEdge::Check)     => Node::Over(Outcome::Showdown(*c, *r, Spot::Checked)),
            (Node::R2(c, r, Spot::Checked), LeducEdge::Raise)     => Node::R2(*c, *r, Spot::CheckRaised),
            (Node::R2(c, r, Spot::Raised), LeducEdge::Call)       => Node::Over(Outcome::Showdown(*c, *r, Spot::Raised)),
            (Node::R2(c, r, Spot::Raised), LeducEdge::Fold)       => Node::Over(Outcome::FoldR2(*c, *r, 1)),
            (Node::R2(c, r, Spot::CheckRaised), LeducEdge::Call)  => Node::Over(Outcome::Showdown(*c, *r, Spot::CheckRaised)),
            (Node::R2(c, r, Spot::CheckRaised), LeducEdge::Fold)  => Node::Over(Outcome::FoldR2(*c, *r, 0)),
            _ => unreachable!(),
        };
        Self { node, ..*self }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        match (&self.node(), turn) {
            (Node::Over(outcome), LeducTurn::Player(p)) => outcome.payoff(p, self.hole()),
            _ => unreachable!(),
        }
    }

    fn depth(&self) -> usize {
        match &self.node() {
            Node::Start | Node::Dealt | Node::R1(_) => 0,
            _ => 1,
        }
    }

    fn exploitability_root() -> Self {
        Self {
            hole: [Card::ALL[0], Card::ALL[0]],
            node: Node::Start,
        }
    }
}
