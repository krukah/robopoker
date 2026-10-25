/// Convergence tests at 2^16 iterations.
#[cfg(test)]
mod tests {
    use crate::*;
    use mccfr::*;
    use pokerkit::*;
    use subgame::*;

    const N14: usize = 1 << 14;
    const N16: usize = 1 << 16;

    trait RpsEquilibrium {
        fn averaged(&self, turn: RpsTurn, edge: RpsEdge) -> Probability;

        fn equilibrium(&self, tolerance: f32) {
            let gap = RpsNash::gap(|turn, edge| self.averaged(turn, edge));
            assert!(gap < tolerance, "off the (0.4, 0.4, 0.2) equilibrium by {gap:.4}");
        }
    }
    impl<R, W, S> RpsEquilibrium for Tabular<RpsEncoder, R, W, S, 1>
    where
        R: RegretSchedule,
        W: WeightSchedule,
        S: SamplingScheme,
    {
        fn averaged(&self, turn: RpsTurn, edge: RpsEdge) -> Probability {
            Equilibrium::averaged_policy(self.profile(), &turn, &edge)
        }
    }
    const WORLDS: usize = 2;

    impl<P> RpsEquilibrium for WorldProfile<'_, subgame::DepthView<'_, P, 1>>
    where
        P: Profile<F = RpsTurn>,
    {
        fn averaged(&self, turn: RpsTurn, edge: RpsEdge) -> Probability {
            (0..WORLDS)
                .map(World::from)
                .map(|w| turn.game::<1>().world(w))
                .map(|i| Equilibrium::averaged_policy(self, &i, &subgame::DepthEdge::<_, 1>::Game(edge)))
                .sum::<Probability>()
                / WORLDS as Probability
        }
    }

    macro_rules! rps {
        ($S:ident, $R:ident, $W:ident, $tol:expr) => {
            paste::paste! {
                #[test]
                fn [<$S:lower _ $R:lower _ $W:lower>]() {
                    RpsEquilibrium::equilibrium(&Tabular::<RpsEncoder, $R, $W, $S, 1>::default().solve(N16), $tol);
                }
            }
        };
    }

    macro_rules! subgame {
        ($S:ident, $R:ident, $W:ident, $tol:expr) => {
            paste::paste! {
                #[test]
                fn [<subgame_ $S:lower _ $R:lower _ $W:lower>]() {
                    let ref blueprint = Tabular::<RpsEncoder, $R, $W, $S, 1>::default().solve(N14);
                    let root = RpsGame::root();
                    RpsEquilibrium::equilibrium(
                        &SubGameSolver::<_, 1, _>::new(
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
        let e16 = Grower::exploitability(
            &Tabular::<RpsEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N16),
        );
        println!("N16={e16:.4}");
        assert!(e16 < 0.03, "2^16 iters: {e16:.4} >= 0.03");
    }
    #[test]
    fn mcxploitability() {
        let solver = Tabular::<RpsEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N16);
        let exact = Grower::exploitability(&solver);
        let mc = Grower::mxploitability(&solver, 64);
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
