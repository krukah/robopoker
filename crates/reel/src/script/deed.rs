use pokerkit::Probability;

/// The semantic class of an action — the vocabulary that paint, probe
/// termination, and gain attribution speak, so none of them ever see a
/// game-specific edge type. `Raise` carries the pot ratio so the gold-to-red
/// gradient generalizes; `Nth` is the categorical fallback for games with no
/// poker semantics at all.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Deed {
    Fold,
    Check,
    Call,
    Open,
    Raise(Probability),
    Shove,
    Draw,
    Nth(usize),
}

impl Deed {
    /// True if this action closes an exploration line.
    pub fn ends(&self) -> bool {
        matches!(self, Self::Fold | Self::Call)
    }
}
