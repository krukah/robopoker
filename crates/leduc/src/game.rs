use super::*;
use rbp_core::*;
use rbp_mccfr::*;

/// Position within a betting round.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Spot {
    Open,
    Checked,
    Raised,
    CheckRaised,
}

/// Terminal outcome of a Leduc hand.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Outcome {
    FoldR1(usize),
    FoldR2(Card, Spot, usize),
    Showdown(Card, Spot, Spot),
}

/// Game tree node: which phase of the hand are we in?
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
    Start,
    Dealt,
    R1(Spot),
    Deal(Spot),
    R2(Card, Spot, Spot),
    Over(Outcome),
}

/// Game state for Leduc Hold'em.
///
/// Hole cards are dealt at root. The `Node` enum encodes the
/// full game phase with zero invalid states.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeducGame {
    hole: [Card; 2],
    node: Node,
}

impl Spot {
    pub fn raised(&self) -> bool {
        matches!(self, Spot::Raised | Spot::CheckRaised)
    }

    fn actor(&self) -> usize {
        match self {
            Spot::Open | Spot::CheckRaised => 0,
            Spot::Checked | Spot::Raised => 1,
        }
    }
}

impl Outcome {
    fn pot(&self) -> [u8; 2] {
        match self {
            Outcome::FoldR1(who) => {
                let mut p = [1, 1];
                p[1 - who] += 2;
                p
            }
            Outcome::FoldR2(_, r1, who) => {
                let base = if r1.raised() { 3 } else { 1 };
                let mut p = [base, base];
                p[1 - who] += 4;
                p
            }
            Outcome::Showdown(_, r1, r2) => {
                let base = if r1.raised() { 3 } else { 1 };
                let extra = if r2.raised() { 4 } else { 0 };
                [base + extra, base + extra]
            }
        }
    }

    fn payoff(&self, p: usize, hole: [Card; 2]) -> Utility {
        let pot = self.pot();
        match self {
            Outcome::FoldR1(who) | Outcome::FoldR2(_, _, who) => {
                if *who == p {
                    -(pot[p] as Utility)
                } else {
                    pot[*who] as Utility
                }
            }
            Outcome::Showdown(board, _, _) => {
                let rank = board.rank();
                let r0 = hole[0].rank();
                let r1 = hole[1].rank();
                let pair0 = r0 == rank;
                let pair1 = r1 == rank;
                let winner = match (pair0, pair1) {
                    (true, false) => Some(0usize),
                    (false, true) => Some(1usize),
                    _ => match r0.cmp(&r1) {
                        std::cmp::Ordering::Greater => Some(0),
                        std::cmp::Ordering::Less => Some(1),
                        std::cmp::Ordering::Equal => None,
                    },
                };
                match winner {
                    None => 0.0,
                    Some(w) if w == p => pot[1 - p] as Utility,
                    Some(_) => -(pot[p] as Utility),
                }
            }
        }
    }
}

impl LeducGame {
    pub(crate) fn spots(&self) -> (Spot, Option<Spot>) {
        match &self.node {
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
        self.hole[actor] = card;
        self
    }

    pub fn hole_card(&self, actor: usize) -> Card {
        self.hole[actor]
    }

    pub fn board(&self) -> Option<Card> {
        match &self.node {
            Node::R2(c, _, _) => Some(*c),
            Node::Over(Outcome::FoldR2(c, _, _)) => Some(*c),
            Node::Over(Outcome::Showdown(c, _, _)) => Some(*c),
            _ => None,
        }
    }

    pub fn board_rank(&self) -> Option<Rank> {
        self.board().map(|c| c.rank())
    }

    pub fn hole_rank(&self, actor: usize) -> Rank {
        self.hole[actor].rank()
    }

    pub fn deals(&self) -> impl Iterator<Item = Card> + '_ {
        let h0 = !matches!(self.node, Node::Start);
        let h1 = !matches!(self.node, Node::Start | Node::Dealt);
        let board = self.board();
        Card::ALL
            .into_iter()
            .filter(move |c| !h0 || *c != self.hole[0])
            .filter(move |c| !h1 || *c != self.hole[1])
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
        match &self.node {
            Node::Start | Node::Dealt | Node::Deal(_) => LeducTurn::Chance,
            Node::Over(_) => LeducTurn::Terminal,
            Node::R1(spot) | Node::R2(_, _, spot) => LeducTurn::Player(spot.actor()),
        }
    }
    #[rustfmt::skip]
    fn apply(&self, edge: Self::E) -> Self {
        if let (Node::Start, LeducEdge::Deal(c)) = (&self.node, edge) {
            return Self { hole: [c, self.hole[1]], node: Node::Dealt };
        }
        if let (Node::Dealt, LeducEdge::Deal(c)) = (&self.node, edge) {
            return Self { hole: [self.hole[0], c], node: Node::R1(Spot::Open) };
        }
        let node = match (&self.node, edge) {
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
        Self { hole: self.hole, node }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        match (&self.node, turn) {
            (Node::Over(outcome), LeducTurn::Player(p)) => outcome.payoff(p, self.hole),
            _ => unreachable!(),
        }
    }

    fn depth(&self) -> usize {
        match &self.node {
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
