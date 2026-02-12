use rbp_cards::*;
use rbp_core::*;
use serde::Serialize;

/// Messages sent from server to client over WebSocket.
/// All per-hand events include hand number for proper sequencing.
/// This ensures clients can correctly associate events with hands
/// and ignore stale events from previous hands.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Initial connection confirmation with seat assignment.
    Connected { room: String, seat: Position },
    /// A new hand is starting.
    HandStart {
        hand: u64,
        dealer: Position,
        stacks: Vec<Chips>,
    },
    /// Player's hole cards for this hand.
    HoleCards { hand: u64, cards: String },
    /// Community cards revealed (cumulative board state).
    Board {
        hand: u64,
        street: String,
        cards: Vec<String>,
    },
    /// A player took an action.
    Action {
        hand: u64,
        seat: Position,
        action: String,
        pot: Chips,
    },
    /// It's your turn to act.
    Decision {
        hand: u64,
        legal: Vec<String>,
        board: Vec<String>,
        pot: Chips,
    },
    /// Hand reached showdown - all reveals at once.
    Showdown { hand: u64, reveals: Vec<Reveal> },
    /// Hand ended with settlements.
    HandEnd { hand: u64, winners: Vec<Winner> },
}

/// A player's cards revealed at showdown.
#[derive(Clone, Debug, Serialize)]
pub struct Reveal {
    pub seat: Position,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cards: Option<String>,
}

/// A winner's payout at hand end.
#[derive(Clone, Debug, Serialize)]
pub struct Winner {
    pub seat: Position,
    pub amount: Chips,
}

impl ServerMessage {
    pub fn connected(room: &str, seat: Position) -> Self {
        Self::Connected {
            room: room.to_string(),
            seat,
        }
    }
    pub fn hand_start(hand: u64, dealer: Position, stacks: Vec<Chips>) -> Self {
        Self::HandStart {
            hand,
            dealer,
            stacks,
        }
    }
    pub fn hole_cards(hand: u64, hole: Hole) -> Self {
        Self::HoleCards {
            hand,
            cards: hole.to_string(),
        }
    }
    pub fn board(hand: u64, street: Street, board: Hand) -> Self {
        Self::Board {
            hand,
            street: street.to_string().to_lowercase(),
            cards: Vec::<Card>::from(board)
                .into_iter()
                .map(|c| c.to_string())
                .collect(),
        }
    }
    pub fn action(hand: u64, seat: Position, action: &str, pot: Chips) -> Self {
        Self::Action {
            hand,
            seat,
            action: action.to_string(),
            pot,
        }
    }
    pub fn decision(hand: u64, legal: Vec<String>, board: Vec<String>, pot: Chips) -> Self {
        Self::Decision {
            hand,
            legal,
            board,
            pot,
        }
    }
    pub fn showdown(hand: u64, reveals: Vec<Reveal>) -> Self {
        Self::Showdown { hand, reveals }
    }
    pub fn hand_end(hand: u64, winners: Vec<Winner>) -> Self {
        Self::HandEnd { hand, winners }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialize server message")
    }
}
