use crate::*;
use mccfr::*;
use monge::Support;

impl Support for Rank {}
impl CfrSecret for Rank {}

/// Unified information set for Kuhn poker.
///
/// A [`Composite`] of public state (betting history + acting flag) and
/// secret state (hole card rank). Suits are strategically irrelevant —
/// J♠ and J♥ produce identical info sets.
pub type KuhnInfo = Composite<KuhnPublic, Rank>;

/// Namespace for the info routines.
pub(crate) struct Info;

impl Info {
    /// Constructor that mirrors the pre-[`Composite`] ergonomics.
    pub fn kuhn_info(acting: bool, rank: Rank, node: History) -> KuhnInfo {
        Composite::new(KuhnPublic::new(acting, node), rank)
    }
}
