use super::*;
use deuce::*;
use pokerkit::Chips;

/// A player's state at the table.
///
/// Tracks chips, betting status, and hole cards. The `cards` field is private
/// information—in a real game, opponents can't see it. For client-side use,
/// unknown cards can be represented with placeholder values.
///
/// `stake` is chips committed this street; `spent` is chips committed this hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seat {
    state: State,
    stack: Chips,
    stake: Chips,
    spent: Chips,
    cards: Hole,
}

impl From<(Hole, Chips)> for Seat {
    fn from((cards, stack): (Hole, Chips)) -> Self {
        Self {
            cards,
            stack,
            spent: 0,
            stake: 0,
            state: State::Betting,
        }
    }
}

impl Seat {
    /// Chips behind (not committed to pot).
    pub fn stack(&self) -> Chips {
        self.stack
    }
    /// Chips committed this street.
    pub fn stake(&self) -> Chips {
        self.stake
    }
    /// Current betting status.
    pub fn state(&self) -> State {
        self.state
    }
    /// Total chips committed this hand.
    pub fn spent(&self) -> Chips {
        self.spent
    }
    /// Hole cards (private information).
    pub fn cards(&self) -> Hole {
        self.cards
    }
    /// Adds winnings to stack.
    pub fn win(&mut self, win: Chips) {
        *self = Self {
            stack: self.stack() + win,
            ..*self
        };
    }
    /// Commits chips from stack to pot.
    pub fn bet(&mut self, bet: Chips) {
        *self = Self {
            stack: self.stack() - bet,
            stake: self.stake() + bet,
            spent: self.spent() + bet,
            ..*self
        };
    }

    pub fn reset_state(&mut self, state: State) {
        *self = Self { state, ..*self };
    }

    pub fn reset_cards(&mut self, cards: Hole) {
        *self = Self { cards, ..*self };
    }

    pub fn reset_stake(&mut self) {
        *self = Self { stake: 0, ..*self };
    }

    pub fn reset_spent(&mut self) {
        *self = Self { spent: 0, ..*self };
    }

    pub fn reset_stack(&mut self) {
        *self = Self {
            stack: pokerkit::STACK,
            ..*self
        };
    }
}

impl std::fmt::Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} ${:>4} {}", self.state(), self.stack(), self.cards())
    }
}
