use rbp_core::Chips;
use rbp_cards::*;

/// A player's state at the table.
///
/// Tracks chips, betting status, and hole cards. The `cards` field is private
/// information—in a real game, opponents can't see it. For client-side use,
/// unknown cards can be represented with placeholder values.
///
/// # Fields
///
/// - `state` — Betting, Shoving (all-in), or Folding
/// - `stack` — Chips behind (not yet committed)
/// - `stake` — Chips committed this street
/// - `spent` — Total chips committed this hand
/// - `cards` — Hole cards (private)
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
        self.stack += win;
    }
    /// Commits chips from stack to pot.
    pub fn bet(&mut self, bet: Chips) {
        self.stack -= bet;
        self.stake += bet;
        self.spent += bet;
    }
    pub fn reset_state(&mut self, state: State) {
        self.state = state;
    }
    pub fn reset_cards(&mut self, cards: Hole) {
        self.cards = cards;
    }
    pub fn reset_stake(&mut self) {
        self.stake = 0;
    }
    pub fn reset_spent(&mut self) {
        self.spent = 0;
    }
    pub fn reset_stack(&mut self) {
        self.stack = rbp_core::STACK;
    }
}

impl std::fmt::Display for Seat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.state,
            format!("${:>4}", self.stack),
            self.cards
        )
    }
}

/// Player betting status within a hand.
///
/// - `Betting` — Active and can still make decisions
/// - `Shoving` — All-in, no more decisions but still in the pot
/// - `Folding` — Out of the hand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Betting,
    Shoving,
    Folding,
}

impl State {
    /// True if player is still competing for the pot.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Betting | Self::Shoving)
    }
}

impl TryFrom<&str> for State {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "P" => Ok(State::Betting),
            "S" => Ok(State::Shoving),
            "F" => Ok(State::Folding),
            _ => Err(anyhow::anyhow!("invalid state string")),
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Betting => write!(f, "P"),
            State::Shoving => write!(f, "S"),
            State::Folding => write!(f, "F"),
        }
    }
}
