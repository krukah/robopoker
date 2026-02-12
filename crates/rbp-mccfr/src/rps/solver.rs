//! RPS solver and algorithm variant tests.
//!
//! [`RPS`] is a generic RPS solver parameterized by algorithm configuration.
//! This module serves as both a test harness for verifying CFR correctness
//! and a reference for how different algorithm variants compose.
//!
//! # Convergence Results
//!
//! | Sampling | Regret   | Weight         | Iterations | ε    | Variance  |
//! |----------|----------|----------------|------------|------|-----------|
//! | External | Summed   | Const/Lin      | 2^14       | 0.03 | Low       |
//! | External | Summed   | Quad/Exp       | 2^15       | 0.04 | Medium    |
//! | External | Linear   | All variants   | 2^15       | 0.03 | Medium    |
//! | External | CFR+     | Const/Lin/Quad | 2^14       | 0.03 | Low       |
//! | External | CFR+     | Exponential    | 2^15       | 0.05 | Medium    |
//! | External | Pluribus | Const/Lin/Quad | 2^14       | 0.03 | Low       |
//! | External | Pluribus | Exponential    | 2^15       | 0.05 | Medium    |
//! | External | DCFR     | Const/Lin/Quad | 2^14       | 0.03 | Low       |
//! | External | DCFR     | Exponential    | 2^15       | 0.05 | Medium    |
//! | Targeted | CFR+     | Const/Lin/Quad | 2^16       | 0.06 | High      |
//! | Targeted | CFR+     | Exponential    | 2^16       | 0.08 | High      |
//! | Targeted | Pluribus | Const/Lin/Quad | 2^16       | 0.06 | High      |
//! | Targeted | Pluribus | Exponential    | 2^16       | 0.08 | High      |
//! | Targeted | DCFR     | Const/Lin/Quad | 2^16       | 0.08 | Very High |
//! | Targeted | DCFR     | Exponential    | 2^16       | 0.10 | Very High |
//! | Prunable | CFR+     | Const/Lin/Quad | 2^14       | 0.03 | Low       |
//! | Prunable | CFR+     | Exponential    | 2^15       | 0.05 | Medium    |
//! | Prunable | Pluribus | Const/Lin/Quad | 2^14       | 0.03 | Low       |
//! | Prunable | Pluribus | Exponential    | 2^15       | 0.05 | Medium    |
//! | Prunable | DCFR     | Const/Lin/Quad | 2^14       | 0.03 | Low       |
//! | Prunable | DCFR     | Exponential    | 2^15       | 0.05 | Medium    |
//! | Pluribus | CFR+     | Const/Lin/Quad | 2^14       | 0.03 | Low       |
//! | Pluribus | CFR+     | Exponential    | 2^15       | 0.05 | Medium    |
//! | Pluribus | Pluribus | Const/Lin/Quad | 2^14       | 0.03 | Low       |
//! | Pluribus | Pluribus | Exponential    | 2^15       | 0.05 | Medium    |
//! | Pluribus | DCFR     | Const/Lin/Quad | 2^14       | 0.03 | Low       |
//! | Pluribus | DCFR     | Exponential    | 2^15       | 0.05 | Medium    |
//!
//! Note: Prunable/Pluribus sampling behave like External for RPS since regrets
//! never fall below the pruning threshold in this small game.
//!
//! # Non-Working Combinations
//!
//! | Sampling | Regret       | Weight            | Reason                              |
//! |----------|--------------|-------------------|-------------------------------------|
//! | Vanilla  | Any          | Any               | Incompatible with external-sampling |
//! | External | Any          | Exponential(0.99) | Oscillates — forgets history        |
//! | Targeted | Summed/Linear| Any               | Too high variance to converge       |
//!
//! # Excluded Combinations
//!
//! SummedRegret and LinearRegret are excluded from Targeted/Prunable/Pluribus tests:
//! - **Targeted + Summed/Linear**: Variance too high to converge reliably
//! - **Prunable/Pluribus + Summed/Linear**: Behave identically to ExternalSampling
//!   for RPS since regrets never fall below the pruning threshold in this small game
//!
//! # Key Findings
//!
//! - **Best**: CFR+/DCFR/Pluribus regret + External/Prunable + Constant/Linear weight
//! - **Pluribus config**: PluribusRegret + LinearWeight + PluribusSampling (flagship)
//! - **Worst working**: DCFR + Targeted → 2^16 iters, ε=0.08 (4× iters, 2.5× tolerance)
//! - **Exponential**: Works at 0.9999 decay, oscillates at 0.99

use crate::*;
use rbp_core::*;
use std::collections::BTreeMap;
use std::marker::PhantomData;

/// Generic RPS solver parameterized by algorithm variants and iteration count.
///
/// - `R` — [`RegretSchedule`] for regret accumulation/discounting
/// - `W` — [`PolicySchedule`] for strategy weight accumulation
/// - `S` — [`SamplingScheme`] for tree exploration
/// - `N` — Number of training iterations (trees to process)
pub struct RPS<R, W, S, const N: usize>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    pub(super) epochs: usize,
    pub(super) phantom: PhantomData<fn() -> (R, W, S)>,
    pub(super) encounters: BTreeMap<RpsTurn, BTreeMap<RpsEdge, (Probability, Utility, Utility, u32)>>,
}

impl<R, W, S, const N: usize> Default for RPS<R, W, S, N>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    fn default() -> Self {
        Self {
            epochs: 0,
            phantom: PhantomData,
            encounters: BTreeMap::new(),
        }
    }
}

impl<R, W, S, const N: usize> Solver for RPS<R, W, S, N>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    type T = RpsTurn;
    type E = RpsEdge;
    type X = RpsTurn;
    type Y = RpsTurn;
    type I = RpsTurn;
    type G = RpsGame;
    type P = Self;
    type N = Self;
    type R = R;
    type W = W;
    type S = S;
    fn tree_count() -> usize {
        N
    }
    fn batch_size() -> usize {
        CFR_BATCH_SIZE_RPS
    }
    fn encoder(&self) -> &Self::N {
        self
    }
    fn profile(&self) -> &Self::P {
        self
    }
    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert((0., 0., 0., 0))
            .0
    }
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert((0., 0., 0., 0))
            .1
    }
    fn mut_evalue(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert((0., 0., 0., 0))
            .2
    }
    fn mut_counts(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32 {
        &mut self
            .encounters
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert((0., 0., 0., 0))
            .3
    }
    fn advance(&mut self) {
        Profile::increment(self)
    }
}

#[rustfmt::skip]
impl<R, W, S, const N: usize> std::fmt::Display for RPS<R, W, S, N>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Epochs: {}  Exploitability: {:.4}", self.epochs, Solver::exploitability(self))?;
        writeln!(f, "┌──────┬──────┬──────────┬──────────┬──────────┬──────────┐")?;
        writeln!(f, "│ Turn │ Edge │ ∑ Regret │ ∑ Weight │  Instant │  Average │")?;
        writeln!(f, "├──────┼──────┼──────────┼──────────┼──────────┼──────────┤")?;
        for (turn, edges) in &self.encounters {
            for (edge, _) in edges {
                writeln!(
                    f,
                    "│ {:>4} │ {:>4} │ {:>+8.2} │ {:>8.2} │ {:>8.2} │ {:>8.2} │",
                    format!("{:?}", turn),
                    format!("{:?}", edge),
                    self.profile().cum_regret(turn, edge),
                    self.profile().cum_weight(turn, edge),
                    self.profile().iterated(turn, edge),
                    self.profile().averaged(turn, edge),
                )?;
            }
        }
        writeln!(f, "└──────┴──────┴──────────┴──────────┴──────────┴──────────┘")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const N12: usize = 1 << 12; // 4K
    const N14: usize = 1 << 14; // 16K
    const N15: usize = 1 << 15; // 32K
    const N16: usize = 1 << 16; // 64K
    const N17: usize = 1 << 17; // 128K
    const N18: usize = 1 << 18; // 256K

    trait RpsEquilibrium {
        fn averaged(&self, turn: RpsTurn, edge: RpsEdge) -> Probability;
    }
    impl<R, W, S, const N: usize> RpsEquilibrium for RPS<R, W, S, N>
    where
        R: RegretSchedule,
        W: PolicySchedule,
        S: SamplingScheme,
    {
        fn averaged(&self, turn: RpsTurn, edge: RpsEdge) -> Probability {
            Profile::averaged(self.profile(), &turn, &edge)
        }
    }
    impl<P> RpsEquilibrium for SubProfile<'_, P>
    where
        P: Profile<T = RpsTurn, E = RpsEdge, I = RpsTurn>,
    {
        fn averaged(&self, turn: RpsTurn, edge: RpsEdge) -> Probability {
            Profile::averaged(self, &SubInfo::Info(turn), &SubEdge::Inner(edge))
        }
    }
    #[rustfmt::skip]
    fn equilibrium(solver: &impl RpsEquilibrium, tolerance: f32) {
        for turn in [RpsTurn::P1, RpsTurn::P2] {
            let r = solver.averaged(turn, RpsEdge::R);
            let p = solver.averaged(turn, RpsEdge::P);
            let s = solver.averaged(turn, RpsEdge::S);
            assert!((r - 0.40).abs() < tolerance, "{:?} R: {:.4} ≠ 0.40", turn, r);
            assert!((p - 0.40).abs() < tolerance, "{:?} P: {:.4} ≠ 0.40", turn, p);
            assert!((s - 0.20).abs() < tolerance, "{:?} S: {:.4} ≠ 0.20", turn, s);
        }
    }

    // Normal RPS convergence tests
    macro_rules! rps {
        ($S:ident, $R:ident, $W:ident, $N:expr, $E:expr) => {
            paste::paste! {
                #[test]
                fn [<$S:lower _ $R:lower _ $W:lower>]() {
                    equilibrium(&RPS::<$R, $W, $S, $N>::default().solve(), $E);
                }
            }
        };
    }

    // Subgame solver convergence tests
    // Blueprint uses fixed N14 training; subgame varies by iteration count
    // SubSolver internally uses ExternalSampling + LinearRegret + LinearWeight
    macro_rules! subgame {
        ($S:ident, $R:ident, $W:ident, $N:expr, $E:expr) => {
            paste::paste! {
                #[test]
                fn [<subgame_ $S:lower _ $R:lower _ $W:lower>]() {
                    type Blueprint = RPS<$R, $W, $S, N14>;
                    let ref blueprint = Blueprint::default().solve();
                    equilibrium(
                        &SubSolver::<_, _, $N>::new(
                            blueprint,
                            blueprint,
                            RpsTurn::P2,
                            vec![],
                            ManyWorlds::uniform(),
                        )
                        .solve()
                        .into_profile(),
                        $E,
                    );
                }
            }
        };
    }

    // External Sampling variants (full 5×4 matrix)
    #[rustfmt::skip] rps!(ExternalSampling, SummedRegret,     ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, SummedRegret,     LinearWeight,      N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, SummedRegret,     QuadraticWeight,   N15, 0.040);
    #[rustfmt::skip] rps!(ExternalSampling, SummedRegret,     ExponentialWeight, N15, 0.040);
    #[rustfmt::skip] rps!(ExternalSampling, LinearRegret,     ConstantWeight,    N15, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, LinearRegret,     LinearWeight,      N16, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, LinearRegret,     QuadraticWeight,   N15, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, LinearRegret,     ExponentialWeight, N15, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, FlooredRegret,    ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, FlooredRegret,    LinearWeight,      N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, FlooredRegret,    QuadraticWeight,   N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, FlooredRegret,    ExponentialWeight, N15, 0.050);
    #[rustfmt::skip] rps!(ExternalSampling, PluribusRegret,   ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, PluribusRegret,   LinearWeight,      N16, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, PluribusRegret,   QuadraticWeight,   N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, PluribusRegret,   ExponentialWeight, N15, 0.050);
    #[rustfmt::skip] rps!(ExternalSampling, DiscountedRegret, ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, DiscountedRegret, LinearWeight,      N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, DiscountedRegret, QuadraticWeight,   N14, 0.030);
    #[rustfmt::skip] rps!(ExternalSampling, DiscountedRegret, ExponentialWeight, N15, 0.050);

    // Targeted Sampling variants (PluribusRegret excluded: doesn't converge correctly)
    #[rustfmt::skip] rps!(TargetedSampling, FlooredRegret,    ConstantWeight,    N16, 0.060);
    #[rustfmt::skip] rps!(TargetedSampling, FlooredRegret,    LinearWeight,      N16, 0.060);
    #[rustfmt::skip] rps!(TargetedSampling, FlooredRegret,    QuadraticWeight,   N16, 0.060);
    #[rustfmt::skip] rps!(TargetedSampling, FlooredRegret,    ExponentialWeight, N16, 0.080);
    #[rustfmt::skip] rps!(TargetedSampling, DiscountedRegret, ConstantWeight,    N16, 0.080);
    #[rustfmt::skip] rps!(TargetedSampling, DiscountedRegret, LinearWeight,      N16, 0.080);
    #[rustfmt::skip] rps!(TargetedSampling, DiscountedRegret, QuadraticWeight,   N16, 0.080);
    #[rustfmt::skip] rps!(TargetedSampling, DiscountedRegret, ExponentialWeight, N16, 0.100);

    // Prunable Sampling variants (3×4 matrix — behaves like External for RPS since regrets stay above threshold)
    #[rustfmt::skip] rps!(PrunableSampling, FlooredRegret,    ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(PrunableSampling, FlooredRegret,    LinearWeight,      N14, 0.030);
    #[rustfmt::skip] rps!(PrunableSampling, FlooredRegret,    QuadraticWeight,   N14, 0.030);
    #[rustfmt::skip] rps!(PrunableSampling, FlooredRegret,    ExponentialWeight, N15, 0.050);
    #[rustfmt::skip] rps!(PrunableSampling, PluribusRegret,   ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(PrunableSampling, PluribusRegret,   LinearWeight,      N16, 0.030);
    #[rustfmt::skip] rps!(PrunableSampling, PluribusRegret,   QuadraticWeight,   N14, 0.030);
    #[rustfmt::skip] rps!(PrunableSampling, PluribusRegret,   ExponentialWeight, N15, 0.050);
    #[rustfmt::skip] rps!(PrunableSampling, DiscountedRegret, ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(PrunableSampling, DiscountedRegret, LinearWeight,      N14, 0.030);
    #[rustfmt::skip] rps!(PrunableSampling, DiscountedRegret, QuadraticWeight,   N14, 0.030);
    #[rustfmt::skip] rps!(PrunableSampling, DiscountedRegret, ExponentialWeight, N15, 0.050);

    // Pluribus Sampling variants (3×4 matrix — probabilistic pruning with warm-up, flagship configuration)
    #[rustfmt::skip] rps!(PluribusSampling, FlooredRegret,    ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(PluribusSampling, FlooredRegret,    LinearWeight,      N14, 0.030);
    #[rustfmt::skip] rps!(PluribusSampling, FlooredRegret,    QuadraticWeight,   N14, 0.030);
    #[rustfmt::skip] rps!(PluribusSampling, FlooredRegret,    ExponentialWeight, N15, 0.050);
    #[rustfmt::skip] rps!(PluribusSampling, PluribusRegret,   ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(PluribusSampling, PluribusRegret,   LinearWeight,      N16, 0.030);
    #[rustfmt::skip] rps!(PluribusSampling, PluribusRegret,   QuadraticWeight,   N14, 0.030);
    #[rustfmt::skip] rps!(PluribusSampling, PluribusRegret,   ExponentialWeight, N15, 0.050);
    #[rustfmt::skip] rps!(PluribusSampling, DiscountedRegret, ConstantWeight,    N14, 0.030);
    #[rustfmt::skip] rps!(PluribusSampling, DiscountedRegret, LinearWeight,      N14, 0.030);
    #[rustfmt::skip] rps!(PluribusSampling, DiscountedRegret, QuadraticWeight,   N14, 0.030);
    #[rustfmt::skip] rps!(PluribusSampling, DiscountedRegret, ExponentialWeight, N15, 0.050);

    #[test]
    fn exploitability() {
        // FlooredRegret (CFR+) provides more stable convergence than SummedRegret
        // Note: Monte Carlo variance can cause small fluctuations, so we only check
        // that overall trend is decreasing (e16 < e14) and final values are below thresholds
        let e14 = Solver::exploitability(
            &RPS::<FlooredRegret, LinearWeight, ExternalSampling, N14>::default().solve(),
        );
        let e15 = Solver::exploitability(
            &RPS::<FlooredRegret, LinearWeight, ExternalSampling, N15>::default().solve(),
        );
        let e16 = Solver::exploitability(
            &RPS::<FlooredRegret, LinearWeight, ExternalSampling, N16>::default().solve(),
        );
        println!("N14={:.4}  N15={:.4}  N16={:.4}", e14, e15, e16);
        assert!(e16 < e14, "exploitability should decrease overall");
        assert!(e14 < 0.04, "2^14 iters: {:.4} >= 0.04", e14);
        assert!(e15 < 0.03, "2^15 iters: {:.4} >= 0.03", e15);
        assert!(e16 < 0.02, "2^16 iters: {:.4} >= 0.02", e16);
    }

    // Subgame solver tests - N14 baseline, higher for slower-converging variants
    #[rustfmt::skip] subgame!(ExternalSampling, SummedRegret,     LinearWeight,      N14, 0.045);
    #[rustfmt::skip] subgame!(ExternalSampling, LinearRegret,     LinearWeight,      N14, 0.045);
    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,    LinearWeight,      N14, 0.040);
    #[rustfmt::skip] subgame!(ExternalSampling, PluribusRegret,   LinearWeight,      N16, 0.030);
    #[rustfmt::skip] subgame!(ExternalSampling, DiscountedRegret, LinearWeight,      N14, 0.045);
    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,    ConstantWeight,    N16, 0.030);
    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,    QuadraticWeight,   N14, 0.040);
    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,    ExponentialWeight, N16, 0.035);
    #[rustfmt::skip] subgame!(PluribusSampling, PluribusRegret,   LinearWeight,      N16, 0.030);
    #[rustfmt::skip] subgame!(PrunableSampling, FlooredRegret,    LinearWeight,      N14, 0.040);
}
