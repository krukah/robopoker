use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;

/// Events broadcast by Engine to all participants.
/// Each per-hand event includes hand number for proper sequencing.
#[derive(Clone, Debug)]
pub enum Event {
    /// New hand starting with initial state.
    HandStart {
        hand: u64,
        dealer: Position,
        stacks: Vec<Chips>,
    },
    /// Player's private hole cards (sent only to them).
    HoleCards { hand: u64, hole: Hole },
    /// Community cards revealed (cumulative board state).
    Board {
        hand: u64,
        street: Street,
        board: Hand,
    },
    /// A player took an action.
    Action {
        hand: u64,
        seat: Position,
        action: Action,
        pot: Chips,
    },
    /// It's your turn to act.
    Decision { hand: u64, recall: Partial },
    /// Hand reached showdown - card reveal.
    Reveal {
        hand: u64,
        seat: Position,
        hole: Option<Hole>,
    },
    /// Hand ended with settlements.
    HandEnd {
        hand: u64,
        winners: Vec<(Position, Chips)>,
    },
    /// Player disconnected.
    Disconnect(Position),
}

impl Event {
    pub fn action(&self) -> Option<Action> {
        match self {
            Event::Action { action, .. } => Some(*action),
            _ => None,
        }
    }
    pub fn position(&self) -> Option<Position> {
        match self {
            Event::Reveal { seat, .. } => Some(*seat),
            Event::Disconnect(pos) => Some(*pos),
            _ => None,
        }
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Event::HandStart { hand, dealer, .. } => {
                write!(f, "Hand #{} (dealer P{})", hand, dealer)
            }
            Event::HoleCards { hole, .. } => write!(f, "Hole: {}", hole),
            Event::Board { street, board, .. } => write!(f, "{}: {}", street, board),
            Event::Action { seat, action, .. } => write!(f, "P{}: {}", seat, action),
            Event::Decision { recall, .. } => write!(
                f,
                "Your turn: {}",
                recall
                    .head()
                    .legal()
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Event::Reveal {
                seat,
                hole: Some(h),
                ..
            } => write!(f, "P{}: {}", seat, h),
            Event::Reveal {
                seat, hole: None, ..
            } => write!(f, "P{}: mucks", seat),
            Event::HandEnd { winners, .. } => {
                let s = winners
                    .iter()
                    .map(|(p, c)| format!("P{} wins {}", p, c))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Winners: {}", s)
            }
            Event::Disconnect(pos) => write!(f, "P{}: disconnected", pos),
        }
    }
}
