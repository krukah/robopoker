use super::*;
use deuce::*;

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
