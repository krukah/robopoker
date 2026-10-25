/// Convergence tests at 2^18 iterations.
#[cfg(test)]
mod tests {
    use crate::*;
    use mccfr::*;
    use subgame::*;

    const N16: usize = 1 << 16;
    const N18: usize = 1 << 18;

    macro_rules! leduc {
        ($S:ident, $R:ident, $W:ident, $tol:expr) => {
            paste::paste! {
                #[test]
                fn [<$S:lower _ $R:lower _ $W:lower>]() {
                    let solver = Tabular::<LeducEncoder, $R, $W, $S, 1>::default().solve(N18);
                    let e = Grower::exploitability(&solver);
                    println!("{} + {} + {}: exploitability = {:.4}", stringify!($S), stringify!($R), stringify!($W), e);
                    assert!(e < $tol, "exploitability {:.4} >= {:.3}", e, $tol);
                }
            }
        };
    }

    //                                                                  tolerance
    #[rustfmt::skip] leduc!(ExternalSampling, FlooredRegret,    LinearWeight,       0.080);
    #[rustfmt::skip] leduc!(ExternalSampling, DiscountedRegret,   LinearWeight,       0.080);
    #[rustfmt::skip] leduc!(PrunableSampling, FlooredRegret,    LinearWeight,       0.080);

    /// `apply` lands bit-identical to the serial loop it replaced, fed the same batch every step.
    macro_rules! equivalence {
        ($name:ident, $encoder:ty, $batch:expr) => {
            #[test]
            fn $name() {
                let mut fast = Tabular::<$encoder, LinearRegret, LinearWeight, ExternalSampling, $batch>::default();
                let mut slow = Tabular::<$encoder, LinearRegret, LinearWeight, ExternalSampling, $batch>::default();
                for _ in 0..64 {
                    let updates = slow.batch();
                    fast.apply(updates.clone());
                    for ref update in updates {
                        slow.update_regret(update);
                        slow.update_weight(update);
                        slow.update_payoff(update);
                        slow.update_visits(update);
                    }
                    fast.advance();
                    slow.advance();
                    assert_eq!(fast.profile().bits(), slow.profile().bits());
                }
            }
        };
    }

    /// Every accumulator's exact bits, keyed by info and edge.
    trait Bits {
        fn bits(&self) -> std::collections::HashMap<(LeducInfo, LeducEdge), [u32; 4]>;
    }

    impl Bits for Book<LeducInfo> {
        fn bits(&self) -> std::collections::HashMap<(LeducInfo, LeducEdge), [u32; 4]> {
            self.encounters()
                .iter()
                .flat_map(|(info, table)| table.iter().map(move |(edge, e)| ((*info, *edge), e)))
                .map(|(key, e)| {
                    (
                        key,
                        [
                            e.weight().to_bits(),
                            e.regret().to_bits(),
                            e.payoff().to_bits(),
                            e.visits(),
                        ],
                    )
                })
                .collect()
        }
    }

    // One tree folds in place; 128 trees (~400 updates) clear `Store::SHARDED` and shard.
    equivalence!(inline_apply_matches_serial, LeducEncoder, 1);
    equivalence!(sharded_apply_matches_serial, LeducEncoder, 128);

    #[test]
    #[ignore = "slow: full convergence run"]
    fn converged_solution() {
        let solver = Tabular::<LeducEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N16);
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
                    "{:>22} {:>5}  avg={:.3}  inst={:.3}",
                    format!("{info}"),
                    format!("{edge}"),
                    profile.averaged_policy(info, edge),
                    profile.instant_policy(info, edge),
                );
            }
        }
    }

    // subgame tests

    macro_rules! subgame {
        ($S:ident, $R:ident, $W:ident) => {
            paste::paste! {
                #[test]
                fn [<subgame_ $S:lower _ $R:lower _ $W:lower>]() {
                    let ref blueprint = Tabular::<LeducEncoder, $R, $W, $S, 1>::default().solve(N18);
                    let external = LeducTurn::Player(1);
                    let root = LeducGame::root();
                    let prior = blueprint.encoder().baseline(&root, external);
                    let profile = SubGameSolver::<_, 1, _>::new(
                        blueprint,
                        external,
                        prior.partition::<2>(),
                        CfrRecall::new(vec![], root),
                    )
                    .solve(N16)
                    .into_profile();
                    let r = profile.sum_regret();
                    assert!(r < 0.5, "subgame sum_regret {r:.4} >= 0.5");
                }
            }
        };
    }

    #[rustfmt::skip] subgame!(ExternalSampling, FlooredRegret,  LinearWeight);
    #[rustfmt::skip] subgame!(ExternalSampling, DiscountedRegret, LinearWeight);
    #[rustfmt::skip] subgame!(PrunableSampling, FlooredRegret,  LinearWeight);

    /// Subgame with prefix replaying P0's check converges.
    #[test]
    fn subgame_after_check() {
        let ref blueprint =
            Tabular::<LeducEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        let external = LeducTurn::Player(0);
        let root = LeducGame::root();
        let entry = root.apply(LeducEdge::Check);
        let prior = blueprint.encoder().baseline(&entry, external);
        let profile = SubGameSolver::<_, 1, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(Replay::<LeducGame>::trace([LeducEdge::Check]), entry),
        )
        .solve(N16)
        .into_profile();
        let r = profile.sum_regret();
        assert!(r < 0.5, "subgame after check sum_regret {r:.4} >= 0.5");
    }

    /// Subgame with prefix replaying P0's raise converges.
    #[test]
    fn subgame_after_raise() {
        let ref blueprint =
            Tabular::<LeducEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        let external = LeducTurn::Player(0);
        let root = LeducGame::root();
        let entry = root.apply(LeducEdge::Raise);
        let prior = blueprint.encoder().baseline(&entry, external);
        let profile = SubGameSolver::<_, 1, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(Replay::<LeducGame>::trace([LeducEdge::Raise]), entry),
        )
        .solve(N16)
        .into_profile();
        let r = profile.sum_regret();
        assert!(r < 0.5, "subgame after raise sum_regret {r:.4} >= 0.5");
    }

    /// Verify restrict produces valid game states with different cards.
    #[test]
    fn restrict_produces_valid_deals() {
        let encoder = LeducEncoder;
        let external = LeducTurn::Player(1);
        let root = LeducGame::root();
        let belief = encoder.baseline(&root, external).partition::<2>();
        for w in 0..2 {
            let game = encoder.restrict(external, World::from(w), &belief, &root);
            assert_ne!(game.hole_card(0), game.hole_card(1));
        }
    }

    /// Action-conditioned subgame: posterior weighted by blueprint reach.
    #[test]
    fn subgame_with_reach_conditioned_posterior() {
        let ref blueprint =
            Tabular::<LeducEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        let internal = LeducTurn::Player(0);
        let external = LeducTurn::Player(1);
        let root = LeducGame::root();
        let entry = root.apply(LeducEdge::Check).apply(LeducEdge::Raise);
        let prefix = [LeducEdge::Check, LeducEdge::Raise];
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
        let profile = SubGameSolver::<_, 1, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(Replay::<LeducGame>::trace(prefix), entry),
        )
        .solve(N16)
        .into_profile();
        let regret = profile.sum_regret();
        // Threshold loosened from 0.01 after swapping subgame regret schedule from LinearRegret
        // to SummedRegret + adding blueprint-prior warmstart.
        assert!(regret < 0.03, "regret {regret:.6} >= 0.03");
    }

    // depth-limited tests

    #[test]
    fn depth_limited_tree_has_front_nodes() {
        let ref blueprint =
            Tabular::<LeducEncoder, FlooredRegret, LinearWeight, ExternalSampling, 1>::default().solve(N18);
        let encoder = DepthEncoder::<_, 4>::new(blueprint, Prefix::default());
        let profile: DepthView<'_, _, 4> = DepthView::new(DepthSampler::<4>::blueprint(blueprint));
        let root: DepthGame<_, 4> = DepthGame::new(LeducGame::root(), LeducTurn::Player(0), Some(0));
        let tree = TreeBuilder::<_, _, ExternalSampling>::new(&encoder, &profile, root, 0).build();
        let n = tree.n();
        assert!(n > 10, "depth-limited tree should have front nodes: {n}");
    }
}
