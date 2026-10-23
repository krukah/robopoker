// Equilibrium values and their μ + 4σ tolerances are derived in README.md.
#[cfg(test)]
mod tests {
    use crate::*;
    use mccfr::*;
    use subgame::*;

    const N18: usize = 1 << 18;

    macro_rules! kuhn {
        ($S:ident, $R:ident, $W:ident, $tol:expr) => {
            paste::paste! {
                #[test]
                fn [<$S:lower _ $R:lower _ $W:lower>]() {
                    let solver = Tabular::<KuhnEncoder, $R, $W, $S, 1>::default().solve(N18);
                    let e = Grower::exploitability(&solver);
                    println!("{} + {} + {}: exploitability = {:.4}", stringify!($S), stringify!($R), stringify!($W), e);
                    assert!(e < $tol, "exploitability {:.4} >= {:.3}", e, $tol);
                }
            }
        };
    }

    impl Info {
        fn view(rank: Rank, node: History) -> KuhnInfo {
            Self::kuhn_info(true, rank, node)
        }
    }

    /// Regression guard: `sampling_strategy` must be a true probability distribution (Σ ≈
    /// 1), so `sampling_reach` uses the same probability `WeightedIndex` draws from.
    #[test]
    fn sampling_strategy_is_normalized() {
        let solver = Tabular::<KuhnEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(1 << 12);
        let p = solver.profile();
        for rank in [Rank::J, Rank::Q, Rank::K] {
            for node in [History::Open, History::Bet, History::Check, History::CheckBet] {
                let info = Info::view(rank, node);
                let sum = p.sampling_strategy(&info).iter().map(|(_, q)| *q).sum::<f32>();
                assert!((sum - 1.0).abs() < 1e-4, "sampling_strategy({rank:?}, {node:?}) sums to {sum:.5}, not 1");
            }
        }
    }

    #[test]
    #[rustfmt::skip]
    fn nash_equilibrium() {
        let solver = Tabular::<KuhnEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        let p = solver.profile();
        let policy = |r, h, e: KuhnEdge| p.averaged_policy(&Info::view(r, h), &e);
        let near = |v: f32, target: f32, tol: f32, label: &str| {
            assert!((v - target).abs() < tol, "{label}: {v:.3} not within ±{tol} of {target:.3}");
        };
        let misses = KuhnNash::misses(|info, edge| p.averaged_policy(info, edge), 1.);
        assert!(misses.is_empty(), "{misses:#?}");
        // structural invariant: K opens 3x J's bluff rate
        let k_bet = policy(Rank::K, History::Open, KuhnEdge::Bet);
        let j_bet = policy(Rank::J, History::Open, KuhnEdge::Bet);
        near(k_bet / j_bet, 3.0, 0.4, "k/a ≈ 3");
    }

    #[test]
    #[ignore = "slow: full convergence run"]
    #[rustfmt::skip]
    fn converged_solution() {
        let solver = Tabular::<KuhnEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        let profile = solver.profile();
        println!("\n=== Opening Ranges ===");
        for (info, edges) in solver.profile().encounters() {
            let s = format!("{info}");
            if s == "J|" || s == "Q|" || s == "K|" {
                for edge in edges.keys() {
                    println!("  {} {:>5}  avg={:.3}", s, edge, profile.averaged_policy(info, edge));
                }
            }
        }
        println!("\n=== Full Solution ===");
        for (info, edges) in solver.profile().encounters() {
            for edge in edges.keys() {
                println!(
                    "{:>12} {:>5}  avg={:.3}  inst={:.3}",
                    format!("{info}"), format!("{edge}"),
                    profile.averaged_policy(info, edge), profile.instant_policy(info, edge),
                );
            }
        }
    }

    //                                                                  tolerance
    #[rustfmt::skip] kuhn!(ExternalSampling, SummedRegret,     ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, SummedRegret,     LinearWeight,       0.025);
    #[rustfmt::skip] kuhn!(ExternalSampling, SummedRegret,     QuadraticWeight,    0.025);
    #[rustfmt::skip] kuhn!(ExternalSampling, SummedRegret,     ExponentialWeight,  0.030);
    #[rustfmt::skip] kuhn!(ExternalSampling, LinearRegret,     ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, LinearRegret,     LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, LinearRegret,     QuadraticWeight,    0.030);
    #[rustfmt::skip] kuhn!(ExternalSampling, LinearRegret,     ExponentialWeight,  0.025);
    #[rustfmt::skip] kuhn!(ExternalSampling, FlooredRegret,    ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, FlooredRegret,    LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, FlooredRegret,    QuadraticWeight,    0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, FlooredRegret,    ExponentialWeight,  0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, AsymmetricRegret, ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, AsymmetricRegret, LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, AsymmetricRegret, QuadraticWeight,    0.035);
    #[rustfmt::skip] kuhn!(ExternalSampling, AsymmetricRegret, ExponentialWeight,  0.030);
    #[rustfmt::skip] kuhn!(ExternalSampling, DiscountedRegret, ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, DiscountedRegret, LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, DiscountedRegret, QuadraticWeight,    0.020);
    #[rustfmt::skip] kuhn!(ExternalSampling, DiscountedRegret, ExponentialWeight,  0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, FlooredRegret,    ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, FlooredRegret,    LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, FlooredRegret,    QuadraticWeight,    0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, FlooredRegret,    ExponentialWeight,  0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, AsymmetricRegret, ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, AsymmetricRegret, LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, AsymmetricRegret, QuadraticWeight,    0.030);
    #[rustfmt::skip] kuhn!(PrunableSampling, AsymmetricRegret, ExponentialWeight,  0.025);
    #[rustfmt::skip] kuhn!(PrunableSampling, DiscountedRegret, ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, DiscountedRegret, LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, DiscountedRegret, QuadraticWeight,    0.020);
    #[rustfmt::skip] kuhn!(PrunableSampling, DiscountedRegret, ExponentialWeight,  0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, FlooredRegret,    ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, FlooredRegret,    LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, FlooredRegret,    QuadraticWeight,    0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, FlooredRegret,    ExponentialWeight,  0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, AsymmetricRegret, ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, AsymmetricRegret, LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, AsymmetricRegret, QuadraticWeight,    0.035);
    #[rustfmt::skip] kuhn!(PluribusSampling, AsymmetricRegret, ExponentialWeight,  0.035);
    #[rustfmt::skip] kuhn!(PluribusSampling, DiscountedRegret, ConstantWeight,     0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, DiscountedRegret, LinearWeight,       0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, DiscountedRegret, QuadraticWeight,    0.020);
    #[rustfmt::skip] kuhn!(PluribusSampling, DiscountedRegret, ExponentialWeight,  0.020);

    // SubGameSolver solves one random deal across alternative worlds.

    const N16: usize = 1 << 16;

    const WORLDS: usize = 2;

    impl Info {
        fn subpolicy(
            profile: &WorldProfile<'_, subgame::DepthView<'_, Book<KuhnInfo>, 1>>,
            rank: Rank,
            node: History,
            edge: KuhnEdge,
        ) -> pokerkit::Probability {
            let info = Info::view(rank, node);
            (0..WORLDS)
                .map(|w| {
                    let fi = info.game::<1>();
                    let wi = fi.world(World::from(w));
                    let fe = subgame::DepthEdge::<_, 1>::Game(edge);
                    Equilibrium::averaged_policy(profile, &wi, &fe)
                })
                .sum::<pokerkit::Probability>()
                / WORLDS as pokerkit::Probability
        }

        #[rustfmt::skip]
        fn subgame_nash(profile: &WorldProfile<'_, subgame::DepthView<'_, Book<KuhnInfo>, 1>>) {
            assert!(Self::subpolicy(profile, Rank::K, History::Bet,      KuhnEdge::Call)  > 0.90,  "K|B  should call");
            assert!(Self::subpolicy(profile, Rank::K, History::CheckBet, KuhnEdge::Call)  > 0.90,  "K|XB should call");
        }
    }

    macro_rules! subgame {
        ($S:ident, $R:ident, $W:ident) => {
            paste::paste! {
                #[test]
                fn [<subgame_ $S:lower _ $R:lower _ $W:lower>]() {
                    let ref blueprint = Tabular::<KuhnEncoder, $R, $W, $S, 1>::default().solve(N18);
                    let external = KuhnTurn::Player(1);
                    let root = KuhnGame::root();
                    let prior = blueprint.encoder().baseline(&root, external);
                    let profile = SubGameSolver::<_, 1, _>::new(
                        blueprint,
                        external,
                        prior.partition::<2>(),
                        CfrRecall::new(vec![], root),
                    )
                    .solve(N16)
                    .into_profile();
                    Info::subgame_nash(&profile);
                }
            }
        };
    }

    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,    LinearWeight);
    #[rustfmt::skip] subgame!(ExternalSampling, DiscountedRegret,   LinearWeight);
    #[rustfmt::skip] subgame!(ExternalSampling, AsymmetricRegret, LinearWeight);
    #[rustfmt::skip] subgame!(PrunableSampling, FlooredRegret,    LinearWeight);
    #[rustfmt::skip] subgame!(PluribusSampling, FlooredRegret,    LinearWeight);

    /// Subgame with prefix replaying P0's check preserves Nash properties.
    #[test]
    fn subgame_after_check() {
        let ref blueprint =
            Tabular::<KuhnEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        let external = KuhnTurn::Player(0);
        let root = KuhnGame::root();
        let entry = root.apply(KuhnEdge::Check);
        let prior = blueprint.encoder().baseline(&entry, external);
        let profile = SubGameSolver::<_, 1, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(Replay::<KuhnGame>::trace([KuhnEdge::Check]), entry),
        )
        .solve(N16)
        .into_profile();
        Info::subgame_nash(&profile);
    }

    /// Subgame with prefix replaying P0's bet preserves Nash properties.
    #[test]
    fn subgame_after_bet() {
        let ref blueprint =
            Tabular::<KuhnEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        let external = KuhnTurn::Player(0);
        let root = KuhnGame::root();
        let entry = root.apply(KuhnEdge::Bet);
        let prior = blueprint.encoder().baseline(&entry, external);
        let profile = SubGameSolver::<_, 1, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(Replay::<KuhnGame>::trace([KuhnEdge::Bet]), entry),
        )
        .solve(N16)
        .into_profile();
        Info::subgame_nash(&profile);
    }

    #[test]
    #[ignore = "slow: convergence curve sweep"]
    fn subgame_convergence_curve() {
        let n_weak: usize = 1 << 8;
        let n_strong: usize = 1 << 16;
        let ref weak =
            Tabular::<KuhnEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(n_weak);
        let ref strong = Tabular::<KuhnEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        eprintln!("\n=== blueprint exploitability ===");
        eprintln!("  weak   (2^8):  {:.6}", Grower::exploitability(weak));
        eprintln!("  strong (2^18): {:.6}", Grower::exploitability(strong));
        eprintln!("\n=== subgame on WEAK blueprint: regret curve ===");
        eprintln!("{:>8} {:>12}", "iter", "regret");
        eprintln!("{:>8} {:>12}", "----", "------");
        let external = KuhnTurn::Player(1);
        let root = KuhnGame::root();
        let prior = weak.encoder().baseline(&root, external);
        let mut solver =
            SubGameSolver::<_, 1, _>::new(weak, external, prior.partition::<2>(), CfrRecall::new(vec![], root));
        for i in 1..=n_strong {
            solver.step();
            if i.is_power_of_two() {
                eprintln!("{:>8} {:>12.6}", i, solver.profile().sum_regret());
            }
        }
        let profile = solver.profile();
        eprintln!("\n=== final subgame policies (averaged over worlds) ===");
        eprintln!("{:>6} {:>10} {:>6} {:>10} {:>10}", "rank", "node", "edge", "subgame", "nash");
        eprintln!("{:>6} {:>10} {:>6} {:>10} {:>10}", "----", "----", "----", "-------", "----");
        let nash = |r, h, e: KuhnEdge| {
            let p = strong.profile();
            Equilibrium::averaged_policy(p, &Info::view(r, h), &e)
        };
        for rank in [Rank::J, Rank::Q, Rank::K] {
            for node in [History::Open, History::Check, History::Bet, History::CheckBet] {
                for edge in CfrInfo::menu(&Info::view(rank, node)) {
                    eprintln!(
                        "{:>6} {:>10} {:>6} {:>10.4} {:>10.4}",
                        rank,
                        format!("{node:?}"),
                        edge,
                        Info::subpolicy(profile, rank, node, edge),
                        nash(rank, node, edge),
                    );
                }
            }
        }
        eprintln!("\n=== checks ===");
        let regret = profile.sum_regret();
        eprintln!("  regret converged: {regret:.6} (< 0.001)");
        assert!(regret < 0.001, "subgame regret {regret:.6} >= 0.001");
        assert!(Info::subpolicy(profile, Rank::K, History::Bet, KuhnEdge::Call) > 0.90, "K|B call");
        assert!(Info::subpolicy(profile, Rank::K, History::CheckBet, KuhnEdge::Call) > 0.90, "K|XB call");
        eprintln!("  K always calls facing bet ✓");
        eprintln!("\n  note: other policies are deal-specific (uniform belief = one fixed deal)");
        eprintln!("  this is expected — game-level Nash requires a real posterior/belief");
    }

    /// Verify restrict produces valid game states with different cards.
    #[test]
    fn restrict_produces_valid_deals() {
        let encoder = KuhnEncoder;
        let external = KuhnTurn::Player(1);
        let root = KuhnGame::root();
        let belief = encoder.baseline(&root, external).partition::<WORLDS>();
        for w in 0..WORLDS {
            let game = encoder.restrict(external, World::from(w), &belief, &root);
            assert_ne!(game.hole_card(0), game.hole_card(1));
        }
    }

    /// Action-conditioned subgame: posterior weighted by blueprint reach.
    #[test]
    fn subgame_with_reach_conditioned_posterior() {
        let ref blueprint =
            Tabular::<KuhnEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        let internal = KuhnTurn::Player(0);
        let external = KuhnTurn::Player(1);
        let root = KuhnGame::root();
        let entry = root.apply(KuhnEdge::Check).apply(KuhnEdge::Bet);
        let prefix = [KuhnEdge::Check, KuhnEdge::Bet];
        // compute action-conditioned posterior over external's hand
        let prior = Card::ALL
            .into_iter()
            .filter(|&c| c != root.hole_card(0))
            .map(|c| root.with_card(1, c))
            .map(|game| {
                let reach = blueprint.external_reach(game, internal, prefix);
                let secret = game.hole_rank(1);
                (secret, reach)
            })
            .fold(Posterior::default(), |post, (s, r)| post.add(s, r));
        eprintln!("\n=== reach-conditioned posterior ===");
        for (secret, reach) in prior.clone() {
            eprintln!("  {secret:?} → {reach:.4}");
        }
        let belief = prior.partition::<WORLDS>();
        eprintln!("\n=== belief partition ({WORLDS} worlds) ===");
        for w in 0..WORLDS {
            eprintln!("  world {w}: weight={:.4}", belief.weights()[w]);
        }
        let profile = SubGameSolver::<_, 1, _>::new(
            blueprint,
            external,
            belief,
            CfrRecall::new(Replay::<KuhnGame>::trace(prefix), entry),
        )
        .solve(N16)
        .into_profile();
        let regret = profile.sum_regret();
        eprintln!("\n=== subgame results (regret={regret:.6}) ===");
        for rank in [Rank::J, Rank::Q, Rank::K] {
            for node in [History::Open, History::Check, History::Bet, History::CheckBet] {
                for edge in CfrInfo::menu(&Info::view(rank, node)) {
                    eprintln!(
                        "  {} {:>10} {:>6} {:>8.4}",
                        rank,
                        format!("{node:?}"),
                        edge,
                        Info::subpolicy(&profile, rank, node, edge),
                    );
                }
            }
        }
        eprintln!("\n=== structural checks ===");
        assert!(regret < 0.01, "regret {regret:.6} >= 0.01");
        assert!(Info::subpolicy(&profile, Rank::K, History::CheckBet, KuhnEdge::Call) > 0.90, "K|XB call");
        eprintln!("  regret < 0.01 ✓");
        eprintln!("  K|XB calls ✓");
    }
}
