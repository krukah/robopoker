use crate::*;
use mccfr::*;

/// The input side of a network: an infoset as [`Features`], each edge as the output that scores
/// it, and the player whose network answers for the infoset.
pub trait Embedder: CfrSpec {
    fn inputs(&self) -> usize;

    fn outputs(&self) -> usize;

    fn features(&self, info: &InfoOf<Self>) -> Features;

    fn turn(&self, info: &InfoOf<Self>) -> TurnOf<Self>;

    fn index(&self, edge: &EdgeOf<Self>) -> usize;
}
