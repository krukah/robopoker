use crate::*;
use mccfr::*;

/// The outputs an infoset can play: one flag per [`Embedder::outputs`].
#[derive(Debug, Clone, PartialEq)]
pub struct Mask(Vec<bool>);

impl IntoIterator for Mask {
    type Item = bool;
    type IntoIter = std::vec::IntoIter<bool>;

    fn into_iter(self) -> Self::IntoIter {
        let Self(legal) = self;
        legal.into_iter()
    }
}

impl Mask {
    pub fn new<X>(embedder: &X, info: &InfoOf<X>) -> Self
    where
        X: Embedder,
    {
        Self(info.menu().fold(vec![false; embedder.outputs()], |mut legal, e| {
            legal[embedder.index(&e)] = true;
            legal
        }))
    }
}
