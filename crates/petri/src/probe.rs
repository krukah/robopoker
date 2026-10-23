use crate::*;
use kuhn::*;
use mccfr::*;
use pokerkit::*;
use roshambo::*;

/// What the page needs from a game beyond its [`Embedder`]: the infosets to draw, and the
/// equilibrium to draw them against.
pub trait Probe: Embedder + Default {
    type N: Encoding<F = Self::F> + Default + Send + Sync;

    fn infos() -> Vec<InfoOf<Self>>;

    fn label(info: &InfoOf<Self>) -> String;

    fn nash(info: &InfoOf<Self>, edge: &EdgeOf<Self>) -> Probability;

    fn tolerance(info: &InfoOf<Self>) -> Probability;
}

impl Probe for KuhnEmbedder {
    type N = KuhnEncoder;

    fn infos() -> Vec<KuhnInfo> {
        [History::Open, History::Check, History::Bet, History::CheckBet]
            .into_iter()
            .flat_map(|node| [Rank::J, Rank::Q, Rank::K].map(|rank| Composite::new(KuhnPublic::new(true, node), rank)))
            .collect()
    }

    fn label(info: &KuhnInfo) -> String {
        let node = match info.public().subgame().as_slice() {
            [] => "open",
            [KuhnEdge::Check] => "check",
            [KuhnEdge::Bet] => "bet",
            _ => "check-bet",
        };
        format!("{:?} · {node}", info.secret())
    }

    fn nash(info: &KuhnInfo, edge: &KuhnEdge) -> Probability {
        KuhnNash::targets()
            .into_iter()
            .find(|&(rank, node, ..)| Composite::new(KuhnPublic::new(true, node), rank) == *info)
            .map(|(_, _, target, p, _)| if target == *edge { p } else { 1. - p })
            .expect("every kuhn infoset has a target")
    }

    fn tolerance(info: &KuhnInfo) -> Probability {
        KuhnNash::targets()
            .into_iter()
            .find(|&(rank, node, ..)| Composite::new(KuhnPublic::new(true, node), rank) == *info)
            .map(|(.., tolerance)| tolerance)
            .expect("every kuhn infoset has a target")
    }
}

impl Probe for RpsEmbedder {
    type N = RpsEncoder;

    fn infos() -> Vec<RpsTurn> {
        vec![RpsTurn::P1, RpsTurn::P2]
    }

    fn label(info: &RpsTurn) -> String {
        format!("{info:?}")
    }

    fn nash(_: &RpsTurn, edge: &RpsEdge) -> Probability {
        RpsNash::policy(*edge)
    }

    /// The bar `CfrNet` holds in `neural`'s RPS test.
    fn tolerance(_: &RpsTurn) -> Probability {
        0.05
    }
}
