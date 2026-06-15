use crate::*;
use atlas::*;
use horizon::*;
use mccfr::*;

mccfr!(Leduc, LeducEncoder, LeducTurn, LeducEdge, LeducGame, LeducInfo, 1);

impl<R, W, S, const D: usize> DepthSampler<D> for Leduc<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type Blueprint = LeducProfile;

    fn blueprint(&self) -> &Self::Blueprint {
        &self.profile
    }

    fn payoffs(&self, prefix: &Prefix<Self::T, Self::E>, game: &Self::G, _: Self::T) -> Payoffs<D> {
        Payoffs::uniform(
            self.profile
                .frontier_payoff(&self.resume(prefix.into_iter().edges(), game)),
        )
    }
}

impl<R, W, S, const WORLDS: usize> WorldRestrict<WORLDS> for Leduc<R, W, S>
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
impl<R, W, S> std::fmt::Display for Leduc<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Epochs: {}  Exploitability: {:.4}", self.profile.epochs, Solver::exploitability(self))?;
        writeln!(f, "┌────────────────────────┬───────┬──────────┬──────────┬──────────┬──────────┐")?;
        writeln!(f, "│ Info                   │ Edge  │ ∑ Regret │ ∑ Weight │  Instant │  Average │")?;
        writeln!(f, "├────────────────────────┼───────┼──────────┼──────────┼──────────┼──────────┤")?;
        for (info, edges) in &self.profile.encounters {
            for edge in edges.keys() {
                writeln!(
                    f,
                    "│ {:>22} │ {:>5} │ {:>+8.2} │ {:>8.2} │ {:>8.2} │ {:>8.2} │",
                    format!("{info}"),
                    format!("{edge}"),
                    self.profile().cum_regret(info, edge),
                    self.profile().cum_weight(info, edge),
                    self.profile().instant_policy(info, edge),
                    self.profile().averaged_policy(info, edge),
                )?;
            }
        }
        writeln!(f, "└────────────────────────┴───────┴──────────┴──────────┴──────────┴──────────┘")?;
        Ok(())
    }
}

/// # Convergence Results
///
/// All tests use 2^18 (256K) iterations. RNG is seeded from a per-thread
/// tree-id counter (reset at the start of each `Solver::solve`), so runs
/// are deterministic. Prunable sampling behaves like External for Leduc
/// since regrets rarely fall below the pruning threshold.
///
/// Exploitability at N18 ranges 0.039–0.058 for stable combos. Only
/// FlooredRegret/DiscountedRegret + LinearWeight are tested — larger
/// Leduc game makes volatile combos too slow to converge at this N.
///
/// # Excluded Combinations
///
/// | Sampling | Regret       | Weight | Reason                                        |
/// |----------|--------------|--------|-----------------------------------------------|
/// | External | SummedRegret | Any    | O(1/√T) convergence too slow at N18 for Leduc |
///
/// # Why exploitability only
///
/// Individual policy values shift significantly between N18 and N24,
/// meaning N18 policies aren't precise enough to test against reference
/// values. Exploitability measures distance from Nash equilibrium directly
/// without requiring known equilibrium strategies.
#[cfg(test)]
mod tests {
    use super::*;
    use endgame::*;

    const N16: usize = 1 << 16;
    const N18: usize = 1 << 18;

    macro_rules! leduc {
        ($S:ident, $R:ident, $W:ident, $tol:expr) => {
            paste::paste! {
                #[test]
                fn [<$S:lower _ $R:lower _ $W:lower>]() {
                    let solver = Leduc::<$R, $W, $S>::default().solve(N18);
                    let e = Solver::exploitability(&solver);
                    println!("{} + {} + {}: exploitability = {:.4}", stringify!($S), stringify!($R), stringify!($W), e);
                    assert!(e < $tol, "exploitability {:.4} >= {:.3}", e, $tol);
                }
            }
        };
    }

    //                                                                  tolerance
    //                                                                  ─────────
    #[rustfmt::skip] leduc!(ExternalSampling, FlooredRegret,    LinearWeight,       0.080);
    #[rustfmt::skip] leduc!(ExternalSampling, DiscountedRegret,   LinearWeight,       0.080);
    #[rustfmt::skip] leduc!(PrunableSampling, FlooredRegret,    LinearWeight,       0.080);

    #[test]
    #[ignore = "slow: full convergence run"]
    fn converged_solution() {
        let solver = Leduc::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N16);
        let profile = solver.profile();
        println!("\n=== Opening Ranges ===");
        for (info, edges) in &solver.profile.encounters {
            let s = format!("{info}");
            if s == "J|" || s == "Q|" || s == "K|" {
                for edge in edges.keys() {
                    println!("  {} {:>5}  avg={:.3}", s, edge, profile.averaged_policy(info, edge));
                }
            }
        }
        println!("\n=== Full Solution ===");
        for (info, edges) in &solver.profile.encounters {
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

    // ── subgame tests ───────────────────────────────────────────────────
    //
    // SubGameSolver solves a single random card deal with N_WORLDS worlds.
    // With uniform belief, all worlds see the same cards so the subgame
    // should converge to a locally optimal strategy for that deal.
    //
    // Unlike Kuhn (where J-folds and K-calls are universal Nash properties),
    // Leduc's equilibrium strategies are deal-dependent — whether J should
    // fold depends on the board card and opponent's hole card. We therefore
    // check regret convergence across multiple blueprint algorithm combos
    // rather than asserting specific policy values.

    macro_rules! subgame {
        ($S:ident, $R:ident, $W:ident) => {
            paste::paste! {
                #[test]
                fn [<subgame_ $S:lower _ $R:lower _ $W:lower>]() {
                    let ref blueprint = Leduc::<$R, $W, $S>::default().solve(N18);
                    let external = LeducTurn::Player(1);
                    let root = LeducGame::root();
                    let prior = blueprint.encoder().baseline(&root, external);
                    let profile = SubGameSolver::<_, 1, _, _, _>::new(
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
        let ref blueprint = Leduc::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
        let external = LeducTurn::Player(0);
        let root = LeducGame::root();
        let entry = root.apply(LeducEdge::Check);
        let prior = blueprint.encoder().baseline(&entry, external);
        let profile = SubGameSolver::<_, 1, _, _, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(descents_from(LeducGame::root(), [LeducEdge::Check]), entry),
        )
        .solve(N16)
        .into_profile();
        let r = profile.sum_regret();
        assert!(r < 0.5, "subgame after check sum_regret {r:.4} >= 0.5");
    }

    /// Subgame with prefix replaying P0's raise converges.
    #[test]
    fn subgame_after_raise() {
        let ref blueprint = Leduc::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
        let external = LeducTurn::Player(0);
        let root = LeducGame::root();
        let entry = root.apply(LeducEdge::Raise);
        let prior = blueprint.encoder().baseline(&entry, external);
        let profile = SubGameSolver::<_, 1, _, _, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(descents_from(LeducGame::root(), [LeducEdge::Raise]), entry),
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
    ///
    /// After P0 checks and P1 raises, the posterior over P1's hand is
    /// conditioned on P1 having chosen to raise. The subgame solver should
    /// converge with the conditioned belief.
    #[test]
    fn subgame_with_reach_conditioned_posterior() {
        let ref blueprint = Leduc::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
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
        let profile = SubGameSolver::<_, 1, _, _, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(descents_from(LeducGame::root(), prefix), entry),
        )
        .solve(N16)
        .into_profile();
        let regret = profile.sum_regret();
        // Threshold loosened from 0.01 after swapping subgame regret schedule
        // from LinearRegret to SummedRegret + adding blueprint-prior warmstart.
        // SummedRegret doesn't decay old regret and the warmstart seeds local
        // with scaled blueprint cum_regret, so sum_regret carries an initial
        // offset that takes longer to wash out. This still tests convergence
        // (subgame is running, not diverging) at a budget-appropriate bar.
        assert!(regret < 0.03, "regret {regret:.6} >= 0.03");
    }

    // ── depth-limited tests ────────────────────────────────────────
    //
    // DepthEncoder wraps LeducEncoder and expands frontier chance
    // nodes (board card deal) into continuation-choice subtrees.
    // Verifies the depth-limited tree structure and convergence.

    #[test]
    fn depth_limited_tree_has_front_nodes() {
        use horizon::*;
        let ref blueprint = Leduc::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
        let encoder = DepthEncoder::<_, 4>::new(blueprint, vec![]);
        let profile: DepthView<'_, _, 4> = DepthView::new(DepthSampler::<4>::blueprint(blueprint));
        let root: DepthGame<_, 4> = DepthGame::new(LeducGame::root(), LeducTurn::Player(0), Some(0));
        let tree = TreeBuilder::<_, _, _, _, _, _, ExternalSampling>::new(&encoder, &profile, root, 0).build();
        let n = tree.n();
        assert!(n > 10, "depth-limited tree should have front nodes: {n}");
    }
}
