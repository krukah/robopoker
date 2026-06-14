use crate::records::Hand as HandRecord;
use crate::records::Participant;
use crate::records::Play;
use crate::records::Room as RoomMarker;
use bouncer::Member;
use cowboys::*;
use kicker::Board;
use kicker::Hole;
use pokerkit::*;

/// Complete context for a hand in progress.
/// Tracks everything needed for persistence and replay.
#[derive(Debug, Clone, Default)]
pub struct HandContext {
    id: ID<HandRecord>,
    hand_number: u64,
    dealer: Position,
    seats: Vec<(Hole, Chips)>,
    actions: Vec<(Position, Action, Option<i32>)>,
    pnl: Vec<Chips>,
}

impl HandContext {
    /// Creates context for a new hand from the current game state.
    pub fn new(hand_number: u64, game: &Game) -> Self {
        let n = game.seats().len();
        Self {
            id: ID::default(),
            hand_number,
            dealer: game.dealer().position(),
            seats: game
                .seats()
                .iter()
                .map(|s| (s.cards(), s.stack() + s.stake()))
                .collect(),
            actions: Vec::new(),
            pnl: vec![0; n],
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
    pub fn actions(&self) -> &[(Position, Action, Option<i32>)] {
        &self.actions
    }
    /// Records a player action.
    pub fn record(&mut self, pos: Position, action: Action, elapsed: Option<i32>) {
        self.actions.push((pos, action, elapsed));
    }
    /// Stores the known PnL for a seat.
    pub fn set_pnl(&mut self, seat: Position, pnl: Chips) {
        self.pnl[seat] = pnl;
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
            .map(|(i, (hole, stack))| Participant::new(hand, f(i), i, *hole, *stack, self.pnl[i]))
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
            .map(|(i, (pos, action, elapsed))| Play::new(hand, i as Epoch, f(*pos), *action, *elapsed))
            .collect()
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
        ctx.record(0, Action::Fold, None);
        ctx.record(1, Action::Check, Some(42));
        assert_eq!(ctx.actions().len(), 2);
        assert_eq!(ctx.actions()[0], (0, Action::Fold, None));
        assert_eq!(ctx.actions()[1], (1, Action::Check, Some(42)));
    }
}
