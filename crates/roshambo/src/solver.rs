use crate::*;
use atlas::*;
use horizon::*;
use mccfr::*;

mccfr!(
    Rps,        //
    RpsEncoder, //
    RpsTurn,    //
    RpsEdge,    //
    RpsGame,    //
    RpsTurn,    //
    1
);

impl<R, W, S, const L: usize> DepthSampler<L> for Rps<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type Blueprint = RpsProfile;

    fn blueprint(&self) -> &Self::Blueprint {
        &self.profile
    }

    fn payoffs(&self, prefix: &Prefix<Self::T, Self::E>, game: &Self::G, _: Self::T) -> Payoffs<L> {
        Payoffs::uniform(
            self.profile
                .frontier_payoff(&self.resume(prefix.into_iter().edges(), game)),
        )
    }
}

impl<R, W, S, const WORLDS: usize> WorldRestrict<WORLDS> for Rps<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn restrict(
        &self,
        external: Self::T,
        world: World,
        belief: &Belief<Secret<Self>, WORLDS>,
        observed: &Self::G,
    ) -> Self::G {
        self.encoder.restrict(external, world, belief, observed)
    }
}

#[rustfmt::skip]
impl<R, W, S> std::fmt::Display for Rps<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Epochs: {}  Exploitability: {:.4}", self.profile.epochs, Solver::exploitability(self))?;
        writeln!(f, "┌──────┬──────┬──────────┬──────────┬──────────┬──────────┐")?;
        writeln!(f, "│ Turn │ Edge │ ∑ Regret │ ∑ Weight │  Instant │  Average │")?;
        writeln!(f, "├──────┼──────┼──────────┼──────────┼──────────┼──────────┤")?;
        for (turn, edges) in &self.profile.encounters {
            for edge in edges.keys() {
                writeln!(
                    f,
                    "│ {:>4} │ {:>4} │ {:>+8.2} │ {:>8.2} │ {:>8.2} │ {:>8.2} │",
                    format!("{turn:?}"),
                    format!("{edge:?}"),
                    self.profile().cum_regret(turn, edge),
                    self.profile().cum_weight(turn, edge),
                    self.profile().instant_policy(turn, edge),
                    self.profile().averaged_policy(turn, edge),
                )?;
            }
        }
        writeln!(f, "└──────┴──────┴──────────┴──────────┴──────────┴──────────┘")?;
        Ok(())
    }
}

/// # Convergence Results
///
/// All tests use 2^16 (64K) iterations. RNG is seeded from a per-thread
/// tree-id counter (reset at the start of each `Solver::solve`), so runs
/// are deterministic. Prunable/Pluribus sampling behave like External for
/// RPS since regrets never fall below the pruning threshold in this small
/// game. Targeted sampling is excluded — its per-node importance
/// weighting introduces too much variance.
///
/// Per-test tolerances follow the same empirical volatility pattern as Kuhn,
/// scaled to RPS's 0.050 base. Higher tolerance = higher variance combo.
///
/// | Tolerance | Combos                                           |
/// |-----------|--------------------------------------------------|
/// | 0.050     | Floored/Discounted regret, Constant/Linear weight |
/// | 0.060     | Linear/Summed regret + Quadratic/Linear weight   |
/// | 0.070     | Pluribus/Summed regret + Quadratic/Exponential   |
/// | 0.080     | Pluribus regret + Exponential/Quadratic (worst)  |
///
/// # Non-Working Combinations
///
/// | Sampling | Regret       | Weight            | Reason                              |
/// |----------|--------------|-------------------|-------------------------------------|
/// | Vanilla  | Any          | Any               | Incompatible with external-sampling |
/// | External | Any          | Exponential(0.99) | Oscillates — forgets history        |
/// | Targeted | Any          | Any               | Too high variance for stable tests  |
///
/// # Excluded Combinations
///
/// - **Prunable/Pluribus + Summed/Linear**: Behave identically to ExternalSampling
///   for RPS since regrets never fall below the pruning threshold in this small game
///
/// # Key Findings
///
/// - **Best**: CFR+/DCFR/Pluribus regret + External/Prunable + Constant/Linear weight
/// - **Pluribus config**: DiscountedRegret + LinearWeight + PluribusSampling (flagship)
/// - **Exponential**: Works at 0.9999 decay, oscillates at 0.99
#[cfg(test)]
mod tests {
    use super::*;
    use endgame::*;
    use pokerkit::*;

    const N14: usize = 1 << 14;
    const N16: usize = 1 << 16;

    trait RpsEquilibrium {
        fn averaged(&self, turn: RpsTurn, edge: RpsEdge) -> Probability;
    }
    impl<R, W, S> RpsEquilibrium for Rps<R, W, S>
    where
        R: RegretSchedule,
        W: WeightSchedule,
        S: SamplingScheme,
    {
        fn averaged(&self, turn: RpsTurn, edge: RpsEdge) -> Probability {
            CfrNash::averaged_policy(self.profile(), &turn, &edge)
        }
    }
    const WORLDS: usize = 2;

    impl<P> RpsEquilibrium for WorldProfile<'_, horizon::DepthView<'_, P, 1>>
    where
        P: RefProf<T = RpsTurn, E = RpsEdge, I = RpsTurn>,
    {
        fn averaged(&self, turn: RpsTurn, edge: RpsEdge) -> Probability {
            (0..WORLDS)
                .map(World::from)
                .map(|w| WorldInfo::new(w, horizon::DepthInfo::<_, 1>::Game(turn)))
                .map(|i| CfrNash::averaged_policy(self, &i, &horizon::DepthEdge::<_, 1>::Game(edge)))
                .sum::<Probability>()
                / WORLDS as Probability
        }
    }
    #[rustfmt::skip]
    fn equilibrium(solver: &impl RpsEquilibrium, tolerance: f32) {
        for turn in [RpsTurn::P1, RpsTurn::P2] {
            let r = solver.averaged(turn, RpsEdge::R);
            let p = solver.averaged(turn, RpsEdge::P);
            let s = solver.averaged(turn, RpsEdge::S);
            assert!((r - 0.40).abs() < tolerance, "{turn:?} R: {r:.4} ≠ 0.40");
            assert!((p - 0.40).abs() < tolerance, "{turn:?} P: {p:.4} ≠ 0.40");
            assert!((s - 0.20).abs() < tolerance, "{turn:?} S: {s:.4} ≠ 0.20");
        }
    }

    macro_rules! rps {
        ($S:ident, $R:ident, $W:ident, $tol:expr) => {
            paste::paste! {
                #[test]
                fn [<$S:lower _ $R:lower _ $W:lower>]() {
                    equilibrium(&Rps::<$R, $W, $S>::default().solve(N16), $tol);
                }
            }
        };
    }

    macro_rules! subgame {
        ($S:ident, $R:ident, $W:ident, $tol:expr) => {
            paste::paste! {
                #[test]
                fn [<subgame_ $S:lower _ $R:lower _ $W:lower>]() {
                    let ref blueprint = Rps::<$R, $W, $S>::default().solve(N14);
                    let root = RpsGame::root();
                    equilibrium(
                        &SubGameSolver::<_, 1, _, _, _>::new(
                            blueprint,
                            RpsTurn::P2,
                            blueprint.encoder().baseline().partition::<2>(),
                            CfrRecall::new(vec![], root),
                        )
                        .solve(N16)
                        .into_profile(),
                        $tol,
                    );
                }
            }
        };
    }

    //                                                                  tolerance
    //                                                                  ─────────
    // External Sampling (5×4 matrix)
    #[rustfmt::skip] rps!(ExternalSampling, SummedRegret,     ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(ExternalSampling, SummedRegret,     LinearWeight,       0.060);
    #[rustfmt::skip] rps!(ExternalSampling, SummedRegret,     QuadraticWeight,    0.060);
    #[rustfmt::skip] rps!(ExternalSampling, SummedRegret,     ExponentialWeight,  0.070);
    #[rustfmt::skip] rps!(ExternalSampling, LinearRegret,     ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(ExternalSampling, LinearRegret,     LinearWeight,       0.050);
    #[rustfmt::skip] rps!(ExternalSampling, LinearRegret,     QuadraticWeight,    0.070);
    #[rustfmt::skip] rps!(ExternalSampling, LinearRegret,     ExponentialWeight,  0.060);
    #[rustfmt::skip] rps!(ExternalSampling, FlooredRegret,    ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(ExternalSampling, FlooredRegret,    LinearWeight,       0.050);
    #[rustfmt::skip] rps!(ExternalSampling, FlooredRegret,    QuadraticWeight,    0.050);
    #[rustfmt::skip] rps!(ExternalSampling, FlooredRegret,    ExponentialWeight,  0.050);
    #[rustfmt::skip] rps!(ExternalSampling, AsymmetricRegret, ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(ExternalSampling, AsymmetricRegret, LinearWeight,       0.050);
    #[rustfmt::skip] rps!(ExternalSampling, AsymmetricRegret, QuadraticWeight,    0.080);
    #[rustfmt::skip] rps!(ExternalSampling, AsymmetricRegret, ExponentialWeight,  0.070);
    #[rustfmt::skip] rps!(ExternalSampling, DiscountedRegret,   ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(ExternalSampling, DiscountedRegret,   LinearWeight,       0.050);
    #[rustfmt::skip] rps!(ExternalSampling, DiscountedRegret,   QuadraticWeight,    0.050);
    #[rustfmt::skip] rps!(ExternalSampling, DiscountedRegret,   ExponentialWeight,  0.050);
    // Prunable Sampling (3×4 matrix)
    #[rustfmt::skip] rps!(PrunableSampling, FlooredRegret,    ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(PrunableSampling, FlooredRegret,    LinearWeight,       0.050);
    #[rustfmt::skip] rps!(PrunableSampling, FlooredRegret,    QuadraticWeight,    0.050);
    #[rustfmt::skip] rps!(PrunableSampling, FlooredRegret,    ExponentialWeight,  0.050);
    #[rustfmt::skip] rps!(PrunableSampling, AsymmetricRegret, ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(PrunableSampling, AsymmetricRegret, LinearWeight,       0.050);
    #[rustfmt::skip] rps!(PrunableSampling, AsymmetricRegret, QuadraticWeight,    0.080);
    #[rustfmt::skip] rps!(PrunableSampling, AsymmetricRegret, ExponentialWeight,  0.060);
    #[rustfmt::skip] rps!(PrunableSampling, DiscountedRegret,   ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(PrunableSampling, DiscountedRegret,   LinearWeight,       0.050);
    #[rustfmt::skip] rps!(PrunableSampling, DiscountedRegret,   QuadraticWeight,    0.050);
    #[rustfmt::skip] rps!(PrunableSampling, DiscountedRegret,   ExponentialWeight,  0.050);
    // Pluribus Sampling (3×4 matrix)
    #[rustfmt::skip] rps!(PluribusSampling, FlooredRegret,    ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(PluribusSampling, FlooredRegret,    LinearWeight,       0.050);
    #[rustfmt::skip] rps!(PluribusSampling, FlooredRegret,    QuadraticWeight,    0.050);
    #[rustfmt::skip] rps!(PluribusSampling, FlooredRegret,    ExponentialWeight,  0.050);
    #[rustfmt::skip] rps!(PluribusSampling, AsymmetricRegret, ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(PluribusSampling, AsymmetricRegret, LinearWeight,       0.050);
    #[rustfmt::skip] rps!(PluribusSampling, AsymmetricRegret, QuadraticWeight,    0.080);
    #[rustfmt::skip] rps!(PluribusSampling, AsymmetricRegret, ExponentialWeight,  0.080);
    #[rustfmt::skip] rps!(PluribusSampling, DiscountedRegret,   ConstantWeight,     0.050);
    #[rustfmt::skip] rps!(PluribusSampling, DiscountedRegret,   LinearWeight,       0.050);
    #[rustfmt::skip] rps!(PluribusSampling, DiscountedRegret,   QuadraticWeight,    0.050);
    #[rustfmt::skip] rps!(PluribusSampling, DiscountedRegret,   ExponentialWeight,  0.050);

    #[test]
    fn exploitability() {
        let e16 = Solver::exploitability(&Rps::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N16));
        println!("N16={e16:.4}");
        assert!(e16 < 0.03, "2^16 iters: {e16:.4} >= 0.03");
    }
    #[test]
    fn mcxploitability() {
        let solver = Rps::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N16);
        let exact = Solver::exploitability(&solver);
        let mc = Solver::mxploitability(&solver, 64);
        assert!((exact - mc).abs() < 1e-6, "mc {mc:.6} != exact {exact:.6}");
    }

    // Subgame solver tests (all stable — SubGameSolver uses LinearRegret + LinearWeight internally)
    #[rustfmt::skip] subgame!(ExternalSampling, SummedRegret,     LinearWeight,       0.050);
    #[rustfmt::skip] subgame!(ExternalSampling, LinearRegret,     LinearWeight,       0.050);
    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,    LinearWeight,       0.050);
    #[rustfmt::skip] subgame!(ExternalSampling, AsymmetricRegret, LinearWeight,       0.050);
    #[rustfmt::skip] subgame!(ExternalSampling, DiscountedRegret,   LinearWeight,       0.050);
    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,    ConstantWeight,     0.050);
    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,    QuadraticWeight,    0.050);
    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,    ExponentialWeight,  0.050);
    #[rustfmt::skip] subgame!(PluribusSampling, AsymmetricRegret, LinearWeight,       0.050);
    #[rustfmt::skip] subgame!(PrunableSampling, FlooredRegret,    LinearWeight,       0.050);
}
