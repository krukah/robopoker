use rbp_auth::Member;
use rbp_cards::Board;
use rbp_cards::Hole;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_records::Hand as HandRecord;
use rbp_records::Participant;
use rbp_records::Play;
use rbp_records::Room as RoomMarker;

/// Complete context for a hand in progress.
/// Tracks everything needed for persistence and replay.
/// Replaces the simpler Replay struct with richer functionality.
#[derive(Debug, Clone)]
pub struct HandContext {
    id: ID<HandRecord>,
    hand_number: u64,
    dealer: Position,
    seats: Vec<(Hole, Chips)>,
    actions: Vec<(Position, Action)>,
}

impl HandContext {
    /// Creates context for a new hand from the current game state.
    pub fn new(hand_number: u64, game: &Game) -> Self {
        Self {
            id: ID::default(),
            hand_number,
            dealer: game.dealer().position(),
            seats: game
                .seats()
                .iter()
                .map(|s| (s.cards(), s.stack()))
                .collect(),
            actions: Vec::new(),
        }
    }
    /// Hand identifier for persistence.
    pub fn id(&self) -> ID<HandRecord> {
        self.id
    }
    /// Hand number in the session.
    pub fn hand_number(&self) -> u64 {
        self.hand_number
    }
    /// Dealer position at hand start.
    pub fn dealer(&self) -> Position {
        self.dealer
    }
    /// Initial seats (hole cards, stack) at hand start.
    pub fn seats(&self) -> &[(Hole, Chips)] {
        &self.seats
    }
    /// All actions recorded in the hand.
    pub fn actions(&self) -> &[(Position, Action)] {
        &self.actions
    }
    /// Records a player action.
    pub fn record(&mut self, pos: Position, action: Action) {
        self.actions.push((pos, action));
    }
    /// Converts to Hand record for persistence.
    pub fn to_hand(&self, room_id: ID<RoomMarker>, board: Board, pot: Chips) -> HandRecord {
        HandRecord::new(self.id, room_id, board, pot, self.dealer)
    }
    /// Generates Participant records for persistence.
    pub fn participants<F>(&self, hand: ID<HandRecord>, f: F) -> Vec<Participant>
    where
        F: Fn(Position) -> Option<ID<Member>>,
    {
        self.seats
            .iter()
            .enumerate()
            .map(|(i, (hole, stack))| Participant::new(hand, f(i), i, *hole, *stack))
            .collect()
    }
    /// Generates Play records for persistence.
    pub fn plays<F>(&self, hand: ID<HandRecord>, f: F) -> Vec<Play>
    where
        F: Fn(Position) -> Option<ID<Member>>,
    {
        self.actions
            .iter()
            .enumerate()
            .map(|(i, (pos, action))| Play::new(hand, i as Epoch, f(*pos), *action))
            .collect()
    }
}

impl Default for HandContext {
    fn default() -> Self {
        Self {
            id: ID::default(),
            hand_number: 0,
            dealer: 0,
            seats: Vec::new(),
            actions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_context() {
        let ctx = HandContext::default();
        assert_eq!(ctx.hand_number(), 0);
        assert_eq!(ctx.dealer(), 0);
        assert!(ctx.seats().is_empty());
        assert!(ctx.actions().is_empty());
    }
    #[test]
    fn record_actions() {
        let mut ctx = HandContext::default();
        ctx.record(0, Action::Fold);
        ctx.record(1, Action::Check);
        assert_eq!(ctx.actions().len(), 2);
        assert_eq!(ctx.actions()[0], (0, Action::Fold));
        assert_eq!(ctx.actions()[1], (1, Action::Check));
    }
}
