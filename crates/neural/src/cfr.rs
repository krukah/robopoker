use crate::*;
use mccfr::*;

/// A [`DeepSolver`] for any game: an encoder, a pair of networks, and a sampling scheme.
pub struct DeepCfr<N, P, S>
where
    N: Encoding,
    P: RegretNet<F = N::F> + StrategyNet,
    S: SamplingScheme,
{
    encoder: N,
    profile: DeepProfile<P>,
    sampling: std::marker::PhantomData<fn() -> S>,
}

impl<N, P, S> CfrSpec for DeepCfr<N, P, S>
where
    N: Encoding + Send + Sync,
    P: RegretNet<F = N::F> + StrategyNet + Send + Sync,
    S: SamplingScheme,
{
    type F = N::F;
}

impl<N, P, S> Grower for DeepCfr<N, P, S>
where
    N: Encoding + Send + Sync,
    P: RegretNet<F = N::F> + StrategyNet + Send + Sync,
    S: SamplingScheme,
{
    type N = N;
    type P = DeepProfile<P>;
    type S = S;

    fn encoder(&self) -> &N {
        &self.encoder
    }

    fn profile(&self) -> &DeepProfile<P> {
        &self.profile
    }
}

impl<N, P, S> DeepSolver for DeepCfr<N, P, S>
where
    N: Encoding + Send + Sync,
    P: RegretNet<F = N::F> + StrategyNet + Send + Sync,
    S: SamplingScheme,
{
    type Net = P;

    fn profile_mut(&mut self) -> &mut DeepProfile<P> {
        &mut self.profile
    }
}

impl<N, P, S> Default for DeepCfr<N, P, S>
where
    N: Encoding + Default,
    P: RegretNet<F = N::F> + StrategyNet + Default,
    S: SamplingScheme,
{
    fn default() -> Self {
        Self::new(N::default(), P::default())
    }
}

impl<N, P, S> DeepCfr<N, P, S>
where
    N: Encoding,
    P: RegretNet<F = N::F> + StrategyNet,
    S: SamplingScheme,
{
    pub fn new(encoder: N, net: P) -> Self {
        Self {
            encoder,
            profile: DeepProfile::new(net),
            sampling: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kuhn::*;
    use roshambo::*;

    type KuhnNet = MockNet<KuhnInfo>;

    /// Bounds sit about 1.6x above the worst of 40 seeds at 2^12 (exploitability 0.0096, pure
    /// miss 0.013); per-action mixed checks wander up to 0.09 and are left to exploitability.
    #[test]
    fn kuhn_matches_nash() {
        let solver = DeepCfr::<KuhnEncoder, KuhnNet, ExternalSampling>::default().solve(1 << 12);
        let e = solver.exploitability();
        assert!(e < 0.015, "exploitability {e:.4} >= 0.015");
        let strays = KuhnNash::strays(|info, edge| solver.profile().averaged_policy(info, edge), 0.03);
        assert!(strays.is_empty(), "{strays:#?}");
    }

    /// RPS has no chance, so this run is deterministic: gap 0.0019 at 2^12.
    #[test]
    fn rps_matches_nash() {
        let solver = DeepCfr::<RpsEncoder, MockNet<RpsTurn>, ExternalSampling>::default().solve(1 << 12);
        let gap = RpsNash::gap(|turn, edge| solver.profile().averaged_policy(&turn, &edge));
        assert!(gap < 0.005, "off the (0.4, 0.4, 0.2) equilibrium by {gap:.4}");
    }
}
