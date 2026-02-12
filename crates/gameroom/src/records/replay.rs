use super::*;
use rbp_cards::Hole;
use rbp_core::*;
use rbp_gameplay::Action;

/// In-flight hand recording state.
/// Captures snapshots at hand start and accumulates actions during play.
/// Consumed at hand end to produce Hand, Participant, and Play records.
#[derive(Debug, Clone)]
pub struct Replay {
    id: ID<Hand>,
    dealer: Position,
    seats: Vec<(Hole, Chips)>,
    plays: Vec<(Position, Action)>, // i think position is technically redundant, since it should be forced / implied by Vec<Action>
}

impl Default for Replay {
    fn default() -> Self {
        Self {
            id: ID::default(),
            dealer: 0,
            seats: Vec::new(),
            plays: Vec::new(),
        }
    }
}

impl Replay {
    pub fn new(dealer: Position, seats: Vec<(Hole, Chips)>) -> Self {
        Self {
            id: ID::default(),
            plays: Vec::new(),
            seats,
            dealer,
        }
    }
    pub fn id(&self) -> ID<Hand> {
        self.id
    }
    pub fn dealer(&self) -> Position {
        self.dealer
    }
    pub fn seats(&self) -> &[(Hole, Chips)] {
        &self.seats
    }
    pub fn plays(&self) -> &[(Position, Action)] {
        &self.plays
    }
    pub fn record(&mut self, pos: Position, action: Action) {
        self.plays.push((pos, action));
    }
}
