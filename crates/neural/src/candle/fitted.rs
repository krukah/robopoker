use crate::*;
use candle_core::Result;
use candle_core::Tensor;
use candle_nn::VarMap;
use mccfr::*;

/// An [`Architecture`] read through one [`Head`], so a net's output meaning is fixed by its type.
/// Keeps the `VarMap` that owns its parameters, so it can be saved as safetensors.
pub struct Fitted<A, H>
where
    A: Architecture,
    H: Head,
{
    net: A,
    vars: VarMap,
    head: std::marker::PhantomData<H>,
}

pub type AdvantageNet<A> = Fitted<A, Regret>;
pub type StrategicNet<A> = Fitted<A, Policy>;

impl<A, H> Fitted<A, H>
where
    A: Architecture,
    H: Head,
{
    /// `vars` must hold every parameter of `net`.
    pub fn new(net: A, vars: VarMap) -> Self {
        Self {
            net,
            vars,
            head: std::marker::PhantomData,
        }
    }

    pub fn vars(&self) -> &VarMap {
        &self.vars
    }

    /// `mask` marks each row's legal outputs.
    pub fn forward(&self, x: &Tensor, mask: &Tensor) -> Result<Tensor> {
        self.net.forward(x).and_then(|logits| H::activate(&logits, mask))
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        self.vars.save(path)
    }

    /// Overwrites every parameter from `path`; fails if a name or shape differs.
    pub fn load(mut self, path: &std::path::Path) -> Result<Self> {
        self.vars.load(path)?;
        Ok(self)
    }
}
