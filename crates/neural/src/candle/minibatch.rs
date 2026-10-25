use crate::*;
use candle_core::Device;
use candle_core::Result;
use candle_core::Tensor;
use mccfr::*;
use pokerkit::*;

/// One optimizer step's rows as tensors: [`Features`] `x`, [`Target`]s `y`, each row's [`Mask`],
/// and each row's `t` weight.
pub struct MiniBatch {
    x: Tensor,
    y: Tensor,
    mask: Tensor,
    weight: Tensor,
}

impl MiniBatch {
    /// Weights divide by `mean`, the whole memory's mean `t`. Normalizing per batch instead would
    /// erase the weighting whenever a batch repeats rows.
    pub fn new<X, K>(embedder: &X, device: &Device, rows: &[&Sample<X::F, K>], mean: Utility) -> Result<Self>
    where
        X: Embedder,
        K: Meaning,
    {
        Ok(Self {
            x: Tensor::from_iter(rows.iter().flat_map(|s| embedder.features(s.info())), device)?
                .reshape(((), embedder.inputs()))?,
            y: Tensor::from_iter(rows.iter().flat_map(|s| Target::new(embedder, s.target())), device)?
                .reshape(((), embedder.outputs()))?,
            mask: Tensor::from_iter(rows.iter().flat_map(|s| Mask::new(embedder, s.info())).map(f32::from), device)?
                .reshape(((), embedder.outputs()))?,
            weight: Tensor::from_iter(rows.iter().map(|s| s.weight() / mean), device)?,
        })
    }

    /// The paper's loss, `Σ t · Σ_a (y(I,a) − f(I,a))²` over each row's legal edges, averaged.
    pub fn loss<A, H>(&self, net: &Fitted<A, H>) -> Result<Tensor>
    where
        A: Architecture,
        H: Head,
    {
        net.forward(&self.x, &self.mask)?
            .sub(&self.y)?
            .sqr()?
            .mul(&self.mask)?
            .sum(1)?
            .mul(&self.weight)?
            .mean_all()
    }
}
