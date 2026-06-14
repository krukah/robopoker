use super::*;
use fulcrum::*;
use regret::*;

/// Game tree node: which phase of the hand are we in?
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
    Start,
    Dealt,
    Open,
    Check,
    Bet,
    CheckBet,
    Over(Outcome),
}

/// Terminal outcome of a Kuhn hand.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Outcome {
    Fold(usize),
    Showdown { raised: bool },
}

/// Game state for Kuhn poker.
///
/// Hole cards are dealt at root. The `Node` enum encodes the
/// full game phase with zero invalid states.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KuhnGame {
    hole: [Card; 2],
    node: Node,
}

impl Outcome {
    fn payoff(&self, player: usize, hole: [Card; 2]) -> Utility {
        match self {
            Outcome::Fold(who) => {
                if *who == player {
                    -1.0
                } else {
                    1.0
                }
            }
            Outcome::Showdown { raised } => {
                let stake = if *raised { 2.0 } else { 1.0 };
                match hole[0].rank().cmp(&hole[1].rank()) {
                    std::cmp::Ordering::Greater => {
                        if player == 0 {
                            stake
                        } else {
                            -stake
                        }
                    }
                    std::cmp::Ordering::Less => {
                        if player == 1 {
                            stake
                        } else {
                            -stake
                        }
                    }
                    std::cmp::Ordering::Equal => 0.0,
                }
            }
        }
    }
}

impl KuhnGame {
    pub fn hole_card(&self, actor: usize) -> Card {
        self.hole[actor]
    }

    pub fn hole_rank(&self, actor: usize) -> Rank {
        self.hole[actor].rank()
    }

    pub(crate) fn history(&self) -> History {
        match self.node {
            Node::Open => History::Open,
            Node::Check => History::Check,
            Node::Bet => History::Bet,
            Node::CheckBet => History::CheckBet,
            _ => History::Open,
        }
    }

    pub fn deals(&self) -> impl Iterator<Item = Card> + '_ {
        let h0 = !matches!(self.node, Node::Start);
        let h1 = !matches!(self.node, Node::Start | Node::Dealt);
        Card::ALL
            .into_iter()
            .filter(move |c| !h0 || *c != self.hole[0])
            .filter(move |c| !h1 || *c != self.hole[1])
    }

    pub fn with_card(mut self, actor: usize, card: Card) -> Self {
        self.hole[actor] = card;
        self
    }

    pub fn all_roots() -> impl Iterator<Item = Self> {
        Card::ALL.into_iter().flat_map(|c0| {
            Card::ALL.into_iter().filter(move |c1| *c1 != c0).map(move |c1| Self {
                hole: [c0, c1],
                node: Node::Open,
            })
        })
    }
}

impl CfrGame for KuhnGame {
    type E = KuhnEdge;
    type T = KuhnTurn;

    fn root() -> Self {
        let mut cards = Card::ALL;
        cards.swap(0, rand::random_range(0..6));
        cards.swap(1, rand::random_range(1..6));
        Self {
            hole: [cards[0], cards[1]],
            node: Node::Open,
        }
    }

    fn turn(&self) -> Self::T {
        match &self.node {
            Node::Start | Node::Dealt => KuhnTurn::Chance,
            Node::Over(_) => KuhnTurn::Terminal,
            Node::Open | Node::CheckBet => KuhnTurn::Player(0),
            Node::Check | Node::Bet => KuhnTurn::Player(1),
        }
    }
    #[rustfmt::skip]
    fn apply(&self, edge: Self::E) -> Self {
        if let (Node::Start, KuhnEdge::Deal(c)) = (&self.node, edge) {
            return Self { hole: [c, self.hole[1]], node: Node::Dealt };
        }
        if let (Node::Dealt, KuhnEdge::Deal(c)) = (&self.node, edge) {
            return Self { hole: [self.hole[0], c], node: Node::Open };
        }
        let node = match (&self.node, edge) {
            (Node::Open, KuhnEdge::Check) => Node::Check,
            (Node::Open, KuhnEdge::Bet)   => Node::Bet,
            (Node::Check, KuhnEdge::Check) => Node::Over(Outcome::Showdown { raised: false }),
            (Node::Check, KuhnEdge::Bet)   => Node::CheckBet,
            (Node::Bet, KuhnEdge::Call)    => Node::Over(Outcome::Showdown { raised: true }),
            (Node::Bet, KuhnEdge::Fold)    => Node::Over(Outcome::Fold(1)),
            (Node::CheckBet, KuhnEdge::Call) => Node::Over(Outcome::Showdown { raised: true }),
            (Node::CheckBet, KuhnEdge::Fold) => Node::Over(Outcome::Fold(0)),
            _ => unreachable!(),
        };
        Self { hole: self.hole, node }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        match (&self.node, turn) {
            (Node::Over(outcome), KuhnTurn::Player(p)) => outcome.payoff(p, self.hole),
            _ => unreachable!(),
        }
    }

    fn exploitability_root() -> Self {
        Self {
            hole: [Card::ALL[0], Card::ALL[0]],
            node: Node::Start,
        }
    }
}
