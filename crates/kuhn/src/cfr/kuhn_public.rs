use crate::*;
use mccfr::*;

/// Public component of a Kuhn information set.
///
/// Pairs the current-street betting history with an `acting` bit that
/// distinguishes player nodes from chance nodes that happen to share the
/// same history. Without `acting`, the exploitability tree's chance root
/// would collide with the corresponding Open player node.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct KuhnPublic {
    acting: bool,
    node: History,
}
impl KuhnPublic {
    pub fn new(acting: bool, node: History) -> Self {
        Self { acting, node }
    }

    fn node(&self) -> History {
        self.node
    }
}
impl KuhnPublic {
    /// Current-phase edge history. Inherent rather than a `CfrPublic`
    /// member: the generic key needs identity and a menu, not a transcript.
    pub fn subgame(&self) -> Vec<KuhnEdge> {
        match self.node() {
            History::Open => vec![],
            History::Check => vec![KuhnEdge::Check],
            History::Bet => vec![KuhnEdge::Bet],
            History::CheckBet => vec![KuhnEdge::Check, KuhnEdge::Bet],
        }
    }
}
impl CfrPublic for KuhnPublic {
    type G = KuhnGame;

    fn menu(&self) -> impl Iterator<Item = KuhnEdge> + use<> {
        match self.node() {
            History::Open | History::Check => vec![KuhnEdge::Check, KuhnEdge::Bet],
            History::Bet | History::CheckBet => vec![KuhnEdge::Fold, KuhnEdge::Call],
        }
        .into_iter()
    }

    fn count(&self) -> usize {
        2
    }
}
impl std::fmt::Display for KuhnPublic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for edge in self.subgame() {
            write!(f, "{edge}")?;
        }
        Ok(())
    }
}
