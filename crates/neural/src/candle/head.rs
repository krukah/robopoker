use candle_core::D;
use candle_core::Result;
use candle_core::Tensor;
use candle_nn::ops;
use mccfr::*;

/// What a net's raw outputs mean: [`Regret`] reads them as advantages, [`Policy`] as a
/// distribution over each row's legal outputs.
pub trait Head: Meaning {
    fn activate(logits: &Tensor, mask: &Tensor) -> Result<Tensor>;
}

impl Head for Regret {
    fn activate(logits: &Tensor, _: &Tensor) -> Result<Tensor> {
        Ok(logits.clone())
    }
}

impl Head for Policy {
    /// Softmax over the legal outputs only: illegal logits become `-∞`, so they get exactly zero mass.
    /// Not `softmax_last_dim`: that fused op has no backward pass, so the strategy net would never train.
    fn activate(logits: &Tensor, mask: &Tensor) -> Result<Tensor> {
        mask.ne(f32::default())?
            .where_cond(logits, &Tensor::full(f32::NEG_INFINITY, logits.shape(), logits.device())?)
            .and_then(|masked| ops::softmax(&masked, D::Minus1))
    }
}
