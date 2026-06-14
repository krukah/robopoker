use super::*;
use monge::Support;
use regret::*;

/// Public component of a Leduc information set.
///
/// Carries everything that's observable to all players at a decision point:
/// the current round's betting history (via `Spot`s), the board rank (when
/// revealed), and an `acting` bit that distinguishes player nodes from
/// chance nodes at the same structural position.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct LeducPublic {
    acting: bool,
    board: Option<Rank>,
    r1: Spot,
    r2: Option<Spot>,
}

impl LeducPublic {
    pub fn new(acting: bool, board: Option<Rank>, r1: Spot, r2: Option<Spot>) -> Self {
        Self { acting, board, r1, r2 }
    }
}

impl CfrPublic for LeducPublic {
    type E = LeducEdge;
    type T = LeducTurn;

    fn choices(&self) -> impl Iterator<Item = Self::E> + use<> {
        match self.r2.unwrap_or(self.r1) {
            Spot::Open | Spot::Checked => vec![LeducEdge::Check, LeducEdge::Raise],
            Spot::Raised | Spot::CheckRaised => vec![LeducEdge::Fold, LeducEdge::Call],
        }
        .into_iter()
    }

    fn subgame(&self) -> Vec<Self::E> {
        let mut h = Vec::new();
        match self.r1 {
            Spot::Open => {}
            Spot::Checked => h.push(LeducEdge::Check),
            Spot::Raised => h.push(LeducEdge::Raise),
            Spot::CheckRaised => {
                h.push(LeducEdge::Check);
                h.push(LeducEdge::Raise);
            }
        }
        if self.board.is_some() || self.r2.is_some() {
            match self.r1 {
                Spot::Checked => h.push(LeducEdge::Check),
                Spot::Raised | Spot::CheckRaised => h.push(LeducEdge::Call),
                Spot::Open => {}
            }
        }
        if let Some(r2) = self.r2 {
            match r2 {
                Spot::Open => {}
                Spot::Checked => h.push(LeducEdge::Check),
                Spot::Raised => h.push(LeducEdge::Raise),
                Spot::CheckRaised => {
                    h.push(LeducEdge::Check);
                    h.push(LeducEdge::Raise);
                }
            }
        }
        h
    }
}

impl std::fmt::Display for LeducPublic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(board) = self.board {
            write!(f, "{board:?}|")?;
        }
        for edge in self.subgame() {
            write!(f, "{edge}")?;
        }
        Ok(())
    }
}

impl Support for Rank {}
impl CfrSecret for Rank {}

/// Unified information set for Leduc Hold'em.
///
/// A [`Composite`] of public state (board rank + round Spots) and secret
/// state (hole card rank). Suits are strategically irrelevant.
pub type LeducInfo = Composite<LeducPublic, Rank>;

/// Constructor that mirrors the pre-[`Composite`] ergonomics.
pub fn leduc_info(acting: bool, rank: Rank, board: Option<Rank>, r1: Spot, r2: Option<Spot>) -> LeducInfo {
    Composite::new(LeducPublic::new(acting, board, r1, r2), rank)
}
