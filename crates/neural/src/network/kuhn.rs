use crate::*;
use kuhn::*;
use mccfr::*;

/// Kuhn infosets as a one-hot card and a one-hot betting history.
#[derive(Debug, Default, Clone, Copy)]
pub struct KuhnEmbedder;

impl CfrSpec for KuhnEmbedder {
    type F = KuhnInfo;
}

impl Embedder for KuhnEmbedder {
    fn inputs(&self) -> usize {
        7
    }

    fn outputs(&self) -> usize {
        4
    }

    fn features(&self, info: &KuhnInfo) -> Features {
        let rank = match info.secret() {
            Rank::J => 0,
            Rank::Q => 1,
            Rank::K => 2,
        };
        let node = match info.public().subgame().as_slice() {
            [] => 0,
            [KuhnEdge::Check] => 1,
            [KuhnEdge::Bet] => 2,
            _ => 3,
        };
        (0..3)
            .map(|i| f32::from(i == rank))
            .chain((0..4).map(|i| f32::from(i == node)))
            .collect()
    }

    fn turn(&self, info: &KuhnInfo) -> KuhnTurn {
        KuhnTurn::Player(info.public().subgame().len() % 2)
    }

    fn index(&self, edge: &KuhnEdge) -> usize {
        match edge {
            KuhnEdge::Check => 0,
            KuhnEdge::Bet => 1,
            KuhnEdge::Fold => 2,
            KuhnEdge::Call => 3,
            KuhnEdge::Deal(_) => unreachable!("chance edges have no output"),
        }
    }
}
