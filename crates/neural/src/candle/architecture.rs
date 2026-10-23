use candle_core::Module;
use candle_core::Result;
use candle_nn::VarBuilder;

/// What a [`CfrNet`](crate::CfrNet) fits: a module from `inputs` features to `outputs` values.
/// Every parameter must come from `vb`, so each fit starts from a fresh `VarMap`.
pub trait Architecture: Module + Send + Sync + Sized {
    fn build(vb: VarBuilder, inputs: usize, outputs: usize) -> Result<Self>;
}
