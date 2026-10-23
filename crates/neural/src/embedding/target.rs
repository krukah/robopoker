use crate::*;
use mccfr::*;

/// A per-edge target laid out over a network's outputs: one value per [`Embedder::outputs`], zero
/// wherever the infoset cannot play.
#[derive(Debug, Clone, PartialEq)]
pub struct Target(Vec<f32>);

impl IntoIterator for Target {
    type Item = f32;
    type IntoIter = std::vec::IntoIter<f32>;

    fn into_iter(self) -> Self::IntoIter {
        let Self(values) = self;
        values.into_iter()
    }
}

impl Target {
    pub fn new<X, K>(embedder: &X, target: &Over<EdgeOf<X>, K>) -> Self
    where
        X: Embedder,
        K: Meaning,
    {
        Self(target.iter().fold(vec![0.; embedder.outputs()], |mut values, &(e, y)| {
            values[embedder.index(&e)] = y;
            values
        }))
    }
}
