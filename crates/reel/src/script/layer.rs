use crate::*;
use mccfr::CfrEdge;

/// One rank of the tree: what chance revealed (a board of tokens), the options one seat holds
/// (a menu of real edges), or how the hand settled (the optional terminal band). The flattened
/// layer sequence is the only shape [`crate::Lay`] needs to measure the tree.
///
/// `Board` and `Settle` are the *major* ranks — boundaries that open and close a phase, drawn
/// on a colored band behind their tiles. `Choice` is the minor rank that lives between them.
#[derive(Debug, Clone)]
pub enum Layer<E>
where
    E: CfrEdge,
{
    Board { phase: String, tokens: Vec<Token> },
    Choice { badge: String, heroic: bool, menu: Vec<E> },
    Settle(Tally),
}

impl<E> Layer<E>
where
    E: CfrEdge,
{
    /// Number of tiles this layer spreads along the slot axis.
    pub fn count(&self) -> usize {
        match self {
            Self::Board { tokens, .. } => tokens.len(),
            Self::Choice { menu, .. } => menu.len(),
            Self::Settle(tally) => tally.tokens().len(),
        }
    }

    /// The actor-band label: `?` for chance, the seat badge for choices,
    /// the winning seat for a settlement.
    pub fn tag(&self) -> String {
        match self {
            Self::Board { .. } => "?".into(),
            Self::Choice { badge, .. } => badge.clone(),
            Self::Settle(tally) => tally.badge().into(),
        }
    }

    /// The drawable faces of a major layer — board reveals or settlement
    /// results. Decision layers have none.
    pub fn faces(&self) -> &[Token] {
        match self {
            Self::Board { tokens, .. } => tokens,
            Self::Settle(tally) => tally.tokens(),
            Self::Choice { .. } => &[],
        }
    }

    /// The phase-band caption: the street for a board, the verdict for a
    /// settlement, nothing for a decision.
    pub fn caption(&self) -> &str {
        match self {
            Self::Board { phase, .. } => phase,
            Self::Settle(tally) => tally.verdict(),
            Self::Choice { .. } => "",
        }
    }
}
