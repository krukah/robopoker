use crate::*;
use candle_core::Module;
use candle_core::Result;
use candle_core::Tensor;
use candle_nn::Linear;
use candle_nn::VarBuilder;
use candle_nn::linear;

/// `depth` ReLU layers of `width` units, then a linear layer onto the outputs.
pub struct MLP(Vec<Linear>);

impl Architecture for MLP {
    fn build(vb: VarBuilder, inputs: usize, outputs: usize) -> Result<Self> {
        let w = DeepHyperParams::get().width();
        let d = DeepHyperParams::get().depth();
        std::iter::empty()
            .chain(std::iter::once(inputs))
            .chain(std::iter::repeat_n(w, d))
            .chain(std::iter::once(outputs))
            .collect::<Vec<_>>()
            .windows(2)
            .enumerate()
            .map(|(i, dims)| linear(dims[0], dims[1], vb.pp(i.to_string())))
            .collect::<Result<Vec<_>>>()
            .map(Self)
    }
}

impl Module for MLP {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let (head, body) = self.0.split_last().expect("an mlp has a head");
        body.iter()
            .try_fold(x.clone(), |x, layer| layer.forward(&x)?.relu())
            .and_then(|x| head.forward(&x))
    }
}

impl AsRef<[Linear]> for MLP {
    fn as_ref(&self) -> &[Linear] {
        &self.0
    }
}
