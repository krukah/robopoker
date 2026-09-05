//! The parsed shape of one `.phh` file.
//!
//! PHH stores a hand as a flat TOML table whose `actions` array interleaves
//! dealer events and player decisions. [`Record`] is that table, typed; [`Step`]
//! is one entry of `actions`.
//!
//! Chip amounts stay at the file's **native scale** (`i32`) — Pluribus logs
//! 50/100 blinds and 10,000 stacks, which do not fit `pokerkit::Chips = i16`
//! once a multiway pot goes all-in. Downstream consumers that need engine chips
//! rescale explicitly; nothing here rounds.

use deuce::*;

/// Chip amount at the hand history's native scale.
///
/// Deliberately not `pokerkit::Chips` — see the module docs.
pub type Amount = i32;

/// Chips in halves.
///
/// Settled stacks, not wagers. A two-way split of an odd pot lands on a half
/// chip, and PHH logs it as such (`10112.5`); eight Pluribus hands do exactly
/// that. Wagers are always whole, so only results carry this type.
pub type Halves = i64;

/// One entry of the PHH `actions` array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// `d dh p<i> <cards>` — deal hole cards to a player.
    Deal(usize, Hole),
    /// `d db <cards>` — deal the flop, turn, or river.
    Board(Vec<Card>),
    /// `p<i> f` — fold.
    Fold(usize),
    /// `p<i> cc` — check or call, whichever the state allows.
    CheckCall(usize),
    /// `p<i> cbr <n>` — complete, bet, or raise **to** a total street wager of
    /// `n`. Note this is a to-amount, not an increment.
    BetTo(usize, Amount),
    /// `p<i> sm <cards>` — show at showdown.
    Show(usize, Hole),
    /// `p<i> sm` with cards withheld — muck at showdown, forfeiting the pot.
    Muck(usize),
}

impl Step {
    /// The seat this step belongs to, or `None` for dealer events.
    pub fn seat(&self) -> Option<usize> {
        match self {
            Self::Deal(i, _)
            | Self::Fold(i)
            | Self::CheckCall(i)
            | Self::BetTo(i, _)
            | Self::Show(i, _)
            | Self::Muck(i) => Some(*i),
            Self::Board(_) => None,
        }
    }
    /// True for steps that consume a betting-round turn.
    pub fn is_decision(&self) -> bool {
        matches!(self, Self::Fold(_) | Self::CheckCall(_) | Self::BetTo(_, _))
    }
}

/// One parsed hand history.
#[derive(Debug, Clone)]
pub struct Record {
    variant: String,
    antes: Vec<Amount>,
    blinds: Vec<Amount>,
    min_bet: Amount,
    starting_stacks: Vec<Amount>,
    finishing_stacks: Vec<Halves>,
    players: Vec<String>,
    actions: Vec<Step>,
    hand: Option<u64>,
    source: String,
}

impl Record {
    pub(crate) fn new(
        variant: String,
        antes: Vec<Amount>,
        blinds: Vec<Amount>,
        min_bet: Amount,
        starting_stacks: Vec<Amount>,
        finishing_stacks: Vec<Halves>,
        players: Vec<String>,
        actions: Vec<Step>,
        hand: Option<u64>,
        source: String,
    ) -> Self {
        Self {
            variant,
            antes,
            blinds,
            min_bet,
            starting_stacks,
            finishing_stacks,
            players,
            actions,
            hand,
            source,
        }
    }

    /// PHH variant code — `NT` is no-limit Texas hold'em.
    pub fn variant(&self) -> &str {
        &self.variant
    }

    pub fn antes(&self) -> &[Amount] {
        &self.antes
    }
    /// Forced blinds/straddles indexed by seat. Seat 0 is the small blind in a
    /// multiway game; the button is the last seat.
    pub fn blinds(&self) -> &[Amount] {
        &self.blinds
    }

    pub fn min_bet(&self) -> Amount {
        self.min_bet
    }

    pub fn starting_stacks(&self) -> &[Amount] {
        &self.starting_stacks
    }
    /// Stacks after settlement, as logged, in half-chips. The conformance
    /// oracle checks [`replay`](crate::replay) against this.
    pub fn finishing_stacks(&self) -> &[Halves] {
        &self.finishing_stacks
    }

    pub fn players(&self) -> &[String] {
        &self.players
    }

    pub fn actions(&self) -> &[Step] {
        &self.actions
    }
    /// The `hand = N` index within its session, when present.
    pub fn hand(&self) -> Option<u64> {
        self.hand
    }
    /// Path the record was read from — used to name failures.
    pub fn source(&self) -> &str {
        &self.source
    }
    /// Number of seats.
    pub fn n(&self) -> usize {
        self.starting_stacks.len()
    }
    /// Seat index of the named player, if seated.
    pub fn seat_of(&self, name: &str) -> Option<usize> {
        self.players.iter().position(|p| p == name)
    }
    /// Per-seat chip delta as logged, in half-chips.
    pub fn pnl(&self) -> Vec<Halves> {
        self.finishing_stacks
            .iter()
            .zip(self.starting_stacks.iter())
            .map(|(f, s)| f - Halves::from(*s) * 2)
            .collect()
    }
    /// Hole cards by seat, in deal order.
    pub fn holes(&self) -> Vec<Option<Hole>> {
        let mut holes = vec![None; self.n()];
        for step in &self.actions {
            if let Step::Deal(i, hole) = step
                && *i < holes.len()
            {
                holes[*i] = Some(*hole);
            }
        }
        holes
    }
    /// Board cards in deal order (flop, then turn, then river).
    pub fn board(&self) -> Vec<Card> {
        self.actions
            .iter()
            .filter_map(|s| match s {
                Step::Board(cards) => Some(cards.clone()),
                _ => None,
            })
            .flatten()
            .collect()
    }
}
