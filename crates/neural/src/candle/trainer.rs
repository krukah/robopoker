use crate::*;
use candle_core::DType;
use candle_core::Result;
use candle_core::Tensor;
use candle_nn::AdamW;
use candle_nn::Optimizer;
use candle_nn::ParamsAdamW;
use candle_nn::VarBuilder;
use candle_nn::VarMap;
use mccfr::*;
use pokerkit::*;
use rand::Rng;
use rand::SeedableRng;

/// Turns an [`Embedder`]'s samples into [`Fitted`] nets on a [`Backend`], and reads them back,
/// shaped and fit by [`DeepHyperParams`].
pub struct Trainer<X, B>
where
    X: Embedder,
    B: Backend,
{
    embedder: X,
    backend: B,
}

impl<X, B> Trainer<X, B>
where
    X: Embedder,
    B: Backend,
{
    pub fn new(embedder: X, backend: B) -> Self {
        Self { embedder, backend }
    }

    pub fn embedder(&self) -> &X {
        &self.embedder
    }

    /// One forward pass for one infoset, read back at its legal edges.
    pub fn predict<A, H>(&self, net: &Fitted<A, H>, info: &InfoOf<X>) -> Result<Over<EdgeOf<X>, H>>
    where
        A: Architecture,
        H: Head,
    {
        let row = net
            .forward(
                &Tensor::from_iter(self.embedder.features(info), self.backend.device())?.unsqueeze(0)?,
                &Tensor::from_iter(Mask::new(&self.embedder, info).into_iter().map(f32::from), self.backend.device())?
                    .unsqueeze(0)?,
            )?
            .squeeze(0)?
            .to_vec1::<f32>()?;
        Ok(info.menu().map(|e| (e, row[self.embedder.index(&e)])).collect())
    }

    /// A net shaped by this embedder and [`DeepHyperParams`], with its parameters read from `path`.
    pub fn load<A, H>(&self, path: &std::path::Path) -> Result<Fitted<A, H>>
    where
        A: Architecture,
        H: Head,
    {
        self.fresh()?.load(path)
    }

    /// A net fit by AdamW on uniform minibatches of `rows`, or on all of them when they fit in one
    /// batch, to minimize [`MiniBatch::loss`]. Starts from `prior` only under [`DeepHyperParams::warm`].
    pub fn fit<A, H>(&self, rows: &Reservoir<Sample<X::F, H>>, prior: Option<Fitted<A, H>>) -> Result<Fitted<A, H>>
    where
        A: Architecture,
        H: Head,
    {
        let net = match prior.filter(|_| DeepHyperParams::get().warm()) {
            Some(net) => net,
            None => self.fresh()?,
        };
        let mut adam = AdamW::new(
            net.vars().all_vars(),
            ParamsAdamW {
                lr: DeepHyperParams::get().rate(),
                weight_decay: 0.,
                ..Default::default()
            },
        )?;
        let rows = rows.iter().collect::<Vec<_>>();
        let mean = rows.iter().map(|s| s.weight()).sum::<Utility>() / rows.len().max(1) as Utility;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(rows.len() as u64);
        for _ in 0..if rows.is_empty() { 0 } else { DeepHyperParams::get().steps() } {
            let batch = if rows.len() > DeepHyperParams::get().batch() {
                (0..DeepHyperParams::get().batch())
                    .map(|_| rows[rng.random_range(0..rows.len())])
                    .collect::<Vec<_>>()
            } else {
                rows.clone()
            };
            adam.backward_step(&MiniBatch::new(&self.embedder, self.backend.device(), &batch, mean)?.loss(&net)?)?;
        }
        Ok(net)
    }

    fn fresh<A, H>(&self) -> Result<Fitted<A, H>>
    where
        A: Architecture,
        H: Head,
    {
        let vars = VarMap::new();
        A::build(
            VarBuilder::from_varmap(&vars, DType::F32, self.backend.device()),
            self.embedder.inputs(),
            self.embedder.outputs(),
        )
        .map(|net| Fitted::new(net, vars))
    }
}
