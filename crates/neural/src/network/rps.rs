use crate::*;
use mccfr::*;
use roshambo::*;

/// Rock-paper-scissors infosets as a one-hot player.
#[derive(Debug, Default, Clone, Copy)]
pub struct RpsEmbedder;

impl CfrSpec for RpsEmbedder {
    type F = RpsTurn;
}

impl Embedder for RpsEmbedder {
    fn inputs(&self) -> usize {
        2
    }

    fn outputs(&self) -> usize {
        3
    }

    fn features(&self, info: &RpsTurn) -> Features {
        [RpsTurn::P1, RpsTurn::P2]
            .iter()
            .map(|p| f32::from(info == p))
            .collect()
    }

    fn turn(&self, info: &RpsTurn) -> RpsTurn {
        *info
    }

    fn index(&self, edge: &RpsEdge) -> usize {
        match edge {
            RpsEdge::R => 0,
            RpsEdge::P => 1,
            RpsEdge::S => 2,
        }
    }
}
