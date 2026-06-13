use super::*;
use rbp_mccfr::*;
use rbp_transport::Support;

/// Betting history visible to a player at their decision point.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum History {
    Open,
    Check,
    Bet,
    CheckBet,
}

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
}

impl CfrPublic for KuhnPublic {
    type E = KuhnEdge;
    type T = KuhnTurn;

    fn choices(&self) -> impl Iterator<Item = Self::E> + use<> {
        match self.node {
            History::Open | History::Check => vec![KuhnEdge::Check, KuhnEdge::Bet],
            History::Bet | History::CheckBet => vec![KuhnEdge::Fold, KuhnEdge::Call],
        }
        .into_iter()
    }

    fn subgame(&self) -> Vec<Self::E> {
        match self.node {
            History::Open => vec![],
            History::Check => vec![KuhnEdge::Check],
            History::Bet => vec![KuhnEdge::Bet],
            History::CheckBet => vec![KuhnEdge::Check, KuhnEdge::Bet],
        }
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

impl Support for Rank {}
impl CfrSecret for Rank {}

/// Unified information set for Kuhn poker.
///
/// A [`Composite`] of public state (betting history + acting flag) and
/// secret state (hole card rank). Suits are strategically irrelevant —
/// J♠ and J♥ produce identical info sets.
pub type KuhnInfo = Composite<KuhnPublic, Rank>;

/// Constructor that mirrors the pre-[`Composite`] ergonomics.
pub fn kuhn_info(acting: bool, rank: Rank, node: History) -> KuhnInfo {
    Composite::new(KuhnPublic::new(acting, node), rank)
}
