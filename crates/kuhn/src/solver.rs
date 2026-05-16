use crate::*;
use rbp_depth::*;
use rbp_mccfr::*;
use rbp_world::*;

mccfr!(
    Kuhn,        //
    KuhnEncoder, //
    KuhnTurn,    //
    KuhnEdge,    //
    KuhnGame,    //
    KuhnInfo,    //
    1
);

impl<R, W, S, const D: usize> DepthSampler<D> for Kuhn<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type Blueprint = KuhnProfile;

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

impl<R, W, S, const WORLDS: usize> WorldRestrict<WORLDS> for Kuhn<R, W, S>
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
impl<R, W, S> std::fmt::Display for Kuhn<R, W, S>
where
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Epochs: {}  Exploitability: {:.4}", self.profile.epochs, Solver::exploitability(self))?;
        writeln!(f, "{}", "=".repeat(72))?;
        writeln!(f, "{:<12} {:>5}  {:>+10}  {:>10}  {:>10}  {:>10}", "Info", "Edge", "Regret", "Weight", "Instant", "Average")?;
        writeln!(f, "{}", "-".repeat(72))?;
        for (info, edges) in &self.profile.encounters {
            for (edge, _) in edges {
                writeln!(
                    f,
                    "{:<12} {:>5}  {:>+10.2}  {:>10.2}  {:>10.2}  {:>10.2}",
                    format!("{}", info),
                    format!("{}", edge),
                    self.profile().cum_regret(info, edge),
                    self.profile().cum_weight(info, edge),
                    self.profile().instant_policy(info, edge),
                    self.profile().averaged_policy(info, edge),
                )?;
            }
        }
        writeln!(f, "{}", "=".repeat(72))?;
        Ok(())
    }
}

/// # Convergence Results
///
/// All tests use 2^18 (256K) iterations. 6-card Kuhn poker ({J,Q,K} × {♠,♥})
/// has 12 info sets and 30 deals. Same-rank deals (J♠ vs J♥) produce ties at
/// showdown, shifting the equilibrium from classical 3-card Kuhn.
///
/// Per-test tolerances are calibrated empirically using **μ + 4σ** rounded up
/// to nearest 0.005, targeting <1% failure probability.
///
/// # Nash Equilibrium for 6-Card Kuhn Poker
///
/// ## Pure strategies (invariant to deck size)
///
/// - **J facing bet**: always fold (EV(call) = −12/7 < −1 = EV(fold))
/// - **K facing bet**: always call (worst case ties K, otherwise wins)
/// - **Q opening**: always check (EV(bet) < EV(check) when Q only gets called by K)
/// - **K after check**: always bet (value bet the nuts)
/// - **J facing check-bet**: always fold (analogous to J facing bet)
/// - **K facing check-bet**: always call (analogous to K facing bet)
///
/// ## Mixed strategies (unique, all fractions of 31)
///
/// From any rank's perspective, opponent has: same rank 1/5, each other 2/5.
///
/// | Variable | Info  | Edge | Value | Indifference condition          |
/// |----------|-------|------|-------|---------------------------------|
/// | a        | J\|   | Bet  | 9/31  | P1-Q indiff facing bet: k = 3a |
/// | k        | K\|   | Bet  | 27/31 | P1-Q indiff facing bet: k = 3a |
/// | c₁       | Q\|B  | Call | 17/31 | P1-J indiff at Check + P1-Q    |
/// | c₂       | Q\|XB | Call | 23/31 | P1-J & P1-Q indiff at Check    |
/// | b        | J\|X  | Bet  | 9/31  | P0-Q indiff at CheckBet        |
/// | d        | Q\|X  | Bet  | 8/31  | P0-J & P0-K indiff at Open     |
///
/// Derivation sketch:
/// 1. P1-Q facing bet indiff → k = 3a
/// 2. P1-J indiff at Check → c₂ = (3+5a)/6
/// 3. P1-Q indiff at Check → c₂ = 2k−1
/// 4. Solving (1)-(3): a = 9/31, k = 27/31, c₂ = 23/31
/// 5. P0-Q indiff at CheckBet → 6b+d = 2
/// 6. P0-K indiff at Open → c₁ = b+d
/// 7. P0-J indiff at Open → b = 6c₁−3
/// 8. Solving (5)-(7): b = 9/31, d = 8/31, c₁ = 17/31
///
/// ## Game value
///
/// EV(P0) = -19/465 ≈ -0.041 (P1 has positional advantage).
/// Compare 3-card Kuhn: -1/18 ≈ -0.056. Same-rank ties reduce P0's
/// disadvantage by ~26%.
///
/// Note: P1-Q is *analytically* indifferent at Check (EV(bet) = EV(check)
/// for all d), so d's regret signal is weak and convergence is slow.
/// The test uses a wider tolerance (0.18) for d accordingly.
#[cfg(test)]
mod tests {
    use super::*;
    use rbp_subgame::*;

    const N18: usize = 1 << 18;

    macro_rules! kuhn {
        ($S:ident, $R:ident, $W:ident, $tol:expr) => {
            paste::paste! {
                #[test]
                fn [<$S:lower _ $R:lower _ $W:lower>]() {
                    let solver = Kuhn::<$R, $W, $S>::default().solve(N18);
                    let e = Solver::exploitability(&solver);
                    println!("{} + {} + {}: exploitability = {:.4}", stringify!($S), stringify!($R), stringify!($W), e);
                    assert!(e < $tol, "exploitability {:.4} >= {:.3}", e, $tol);
                }
            }
        };
    }

    fn view(rank: Rank, node: History) -> KuhnInfo {
        kuhn_info(true, rank, node)
    }

    #[test]
    #[rustfmt::skip]
    fn nash_equilibrium() {
        let solver = Kuhn::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
        let p = solver.profile();
        let policy = |r, h, e: KuhnEdge| p.averaged_policy(&view(r, h), &e);
        let near = |v: f32, target: f32, tol: f32, label: &str| {
            assert!((v - target).abs() < tol, "{label}: {v:.3} not within ±{tol} of {target:.3}");
        };
        // ── pure strategies ────────────────────────────────────────────
        assert!(policy(Rank::J, History::Bet,      KuhnEdge::Fold)  > 0.95, "J|B  should fold");
        assert!(policy(Rank::J, History::CheckBet, KuhnEdge::Fold)  > 0.95, "J|XB should fold");
        assert!(policy(Rank::K, History::Bet,      KuhnEdge::Call)  > 0.95, "K|B  should call");
        assert!(policy(Rank::K, History::CheckBet, KuhnEdge::Call)  > 0.95, "K|XB should call");
        assert!(policy(Rank::K, History::Check,    KuhnEdge::Bet)   > 0.95, "K|X  should bet");
        assert!(policy(Rank::Q, History::Open,     KuhnEdge::Check) > 0.85, "Q|   should check");
        // ── mixed strategies (analytical: all fractions of 31) ─────────
        near(policy(Rank::J, History::Open,     KuhnEdge::Bet),   9.0/31.0, 0.05, "a  = J|  bet");
        near(policy(Rank::K, History::Open,     KuhnEdge::Bet),  27.0/31.0, 0.05, "k  = K|  bet");
        near(policy(Rank::Q, History::Bet,      KuhnEdge::Call), 17.0/31.0, 0.08, "c₁ = Q|B call");
        near(policy(Rank::Q, History::CheckBet, KuhnEdge::Call), 23.0/31.0, 0.05, "c₂ = Q|XB call");
        near(policy(Rank::J, History::Check,    KuhnEdge::Bet),   9.0/31.0, 0.05, "b  = J|X bet");
        near(policy(Rank::Q, History::Check,    KuhnEdge::Bet),   8.0/31.0, 0.18, "d  = Q|X bet");
        // ── structural invariant: K opens 3× J's bluff rate ───────────
        let k_bet = policy(Rank::K, History::Open, KuhnEdge::Bet);
        let j_bet = policy(Rank::J, History::Open, KuhnEdge::Bet);
        near(k_bet / j_bet, 3.0, 0.4, "k/a ≈ 3");
    }

    #[test]
    #[ignore]
    #[rustfmt::skip]
    fn converged_solution() {
        let solver = Kuhn::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
        let profile = solver.profile();
        println!("\n=== Opening Ranges ===");
        for (info, edges) in &solver.profile.encounters {
            let s = format!("{}", info);
            if s == "J|" || s == "Q|" || s == "K|" {
                for (edge, _) in edges {
                    println!("  {} {:>5}  avg={:.3}", s, edge, profile.averaged_policy(info, edge));
                }
            }
        }
        println!("\n=== Full Solution ===");
        for (info, edges) in &solver.profile.encounters {
            for (edge, _) in edges {
                println!(
                    "{:>12} {:>5}  avg={:.3}  inst={:.3}",
                    format!("{}", info), format!("{}", edge),
                    profile.averaged_policy(info, edge), profile.instant_policy(info, edge),
                );
            }
        }
    }

    //                                                                  tolerance
    //                                                                  ─────────
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

    // ── subgame tests ───────────────────────────────────────────────────
    //
    // SubGameSolver solves a single random card deal with 4 alternative worlds.
    // With uniform belief, all worlds see the same cards so averaging across
    // worlds recovers the per-deal strategy. Unencountered info sets fall
    // back to blueprint values, so the combined profile approximates Nash.
    //
    // Pure-strategy Nash properties (K calls, J folds, K bets after check)
    // hold regardless of which cards were dealt, providing strong structural
    // correctness checks beyond regret convergence alone.

    const N16: usize = 1 << 16;

    const WORLDS: usize = 2;

    fn subpolicy(
        profile: &WorldProfile<'_, rbp_depth::DepthView<'_, KuhnProfile, 1>>,
        rank: Rank,
        node: History,
        edge: KuhnEdge,
    ) -> rbp_core::Probability {
        let info = view(rank, node);
        (0..WORLDS)
            .map(|w| {
                let fi = rbp_depth::DepthInfo::<_, 1>::Game(info);
                let wi = WorldInfo::new(World::from(w), fi);
                let fe = rbp_depth::DepthEdge::<_, 1>::Game(edge);
                CfrNash::averaged_policy(profile, &wi, &fe)
            })
            .sum::<rbp_core::Probability>()
            / WORLDS as rbp_core::Probability
    }

    #[rustfmt::skip]
    fn subgame_nash(profile: &WorldProfile<'_, rbp_depth::DepthView<'_, KuhnProfile, 1>>) {
        assert!(subpolicy(profile, Rank::K, History::Bet,      KuhnEdge::Call)  > 0.90,  "K|B  should call");
        assert!(subpolicy(profile, Rank::K, History::CheckBet, KuhnEdge::Call)  > 0.90,  "K|XB should call");
    }

    macro_rules! subgame {
        ($S:ident, $R:ident, $W:ident) => {
            paste::paste! {
                #[test]
                fn [<subgame_ $S:lower _ $R:lower _ $W:lower>]() {
                    let ref blueprint = Kuhn::<$R, $W, $S>::default().solve(N18);
                    let external = KuhnTurn::Player(1);
                    let root = KuhnGame::root();
                    let prior = blueprint.encoder().baseline(&root, external);
                    let profile = SubGameSolver::<_, 1, _, _, _>::new(
                        blueprint,
                        external,
                        prior.partition::<2>(),
                        CfrRecall::new(vec![], root),
                    )
                    .solve(N16)
                    .into_profile();
                    subgame_nash(&profile);
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
            Kuhn::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
        let external = KuhnTurn::Player(0);
        let root = KuhnGame::root();
        let entry = root.apply(KuhnEdge::Check);
        let prior = blueprint.encoder().baseline(&entry, external);
        let profile = SubGameSolver::<_, 1, _, _, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(descents_from(KuhnGame::root(), [KuhnEdge::Check]), entry),
        )
        .solve(N16)
        .into_profile();
        subgame_nash(&profile);
    }

    /// Subgame with prefix replaying P0's bet preserves Nash properties.
    #[test]
    fn subgame_after_bet() {
        let ref blueprint =
            Kuhn::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
        let external = KuhnTurn::Player(0);
        let root = KuhnGame::root();
        let entry = root.apply(KuhnEdge::Bet);
        let prior = blueprint.encoder().baseline(&entry, external);
        let profile = SubGameSolver::<_, 1, _, _, _>::new(
            blueprint,
            external,
            prior.partition::<2>(),
            CfrRecall::new(descents_from(KuhnGame::root(), [KuhnEdge::Bet]), entry),
        )
        .solve(N16)
        .into_profile();
        subgame_nash(&profile);
    }

    #[test]
    #[ignore]
    fn subgame_convergence_curve() {
        let n_weak: usize = 1 << 8;
        let n_strong: usize = 1 << 16;
        let ref weak =
            Kuhn::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(n_weak);
        let ref strong =
            Kuhn::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
        eprintln!("\n=== blueprint exploitability ===");
        eprintln!("  weak   (2^8):  {:.6}", Solver::exploitability(weak));
        eprintln!("  strong (2^18): {:.6}", Solver::exploitability(strong));
        eprintln!("\n=== subgame on WEAK blueprint: regret curve ===");
        eprintln!("{:>8} {:>12}", "iter", "regret");
        eprintln!("{:>8} {:>12}", "----", "------");
        let external = KuhnTurn::Player(1);
        let root = KuhnGame::root();
        let prior = weak.encoder().baseline(&root, external);
        let mut solver = SubGameSolver::<_, 1, _, _, _>::new(
            weak,
            external,
            prior.partition::<2>(),
            CfrRecall::new(vec![], root),
        );
        for i in 1..=n_strong {
            solver.step();
            if i.count_ones() == 1 {
                eprintln!("{:>8} {:>12.6}", i, solver.profile().sum_regret());
            }
        }
        let profile = solver.profile();
        eprintln!("\n=== final subgame policies (averaged over worlds) ===");
        eprintln!(
            "{:>6} {:>10} {:>6} {:>10} {:>10}",
            "rank", "node", "edge", "subgame", "nash"
        );
        eprintln!(
            "{:>6} {:>10} {:>6} {:>10} {:>10}",
            "----", "----", "----", "-------", "----"
        );
        let nash = |r, h, e: KuhnEdge| {
            let p = strong.profile();
            CfrNash::averaged_policy(p, &view(r, h), &e)
        };
        for rank in [Rank::J, Rank::Q, Rank::K] {
            for node in [
                History::Open,
                History::Check,
                History::Bet,
                History::CheckBet,
            ] {
                for edge in CfrInfo::choices(&view(rank, node)) {
                    eprintln!(
                        "{:>6} {:>10} {:>6} {:>10.4} {:>10.4}",
                        rank,
                        format!("{:?}", node),
                        edge,
                        subpolicy(profile, rank, node, edge),
                        nash(rank, node, edge),
                    );
                }
            }
        }
        eprintln!("\n=== checks ===");
        let regret = profile.sum_regret();
        eprintln!("  regret converged: {:.6} (< 0.001)", regret);
        assert!(regret < 0.001, "subgame regret {regret:.6} >= 0.001");
        assert!(
            subpolicy(profile, Rank::K, History::Bet, KuhnEdge::Call) > 0.90,
            "K|B call"
        );
        assert!(
            subpolicy(profile, Rank::K, History::CheckBet, KuhnEdge::Call) > 0.90,
            "K|XB call"
        );
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
    ///
    /// After P0 checks and P1 bets, the posterior over P1's hand is conditioned
    /// on P1 having chosen to bet. In Nash, K bets ~100% after check while J/Q
    /// bet less, so the posterior upweights K. The subgame solver should find
    /// that facing a K-heavy range, J should fold at CheckBet.
    #[test]
    fn subgame_with_reach_conditioned_posterior() {
        let ref blueprint =
            Kuhn::<FlooredRegret, LinearWeight, ExternalSampling>::default().solve(N18);
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
        for (secret, reach) in prior.clone().into_iter() {
            eprintln!("  {:?} → {:.4}", secret, reach);
        }
        let belief = prior.partition::<WORLDS>();
        eprintln!("\n=== belief partition ({WORLDS} worlds) ===");
        for w in 0..WORLDS {
            eprintln!("  world {w}: weight={:.4}", belief.weights()[w]);
        }
        let profile = SubGameSolver::<_, 1, _, _, _>::new(
            blueprint,
            external,
            belief,
            CfrRecall::new(descents_from(KuhnGame::root(), prefix), entry),
        )
        .solve(N16)
        .into_profile();
        let regret = profile.sum_regret();
        eprintln!("\n=== subgame results (regret={regret:.6}) ===");
        for rank in [Rank::J, Rank::Q, Rank::K] {
            for node in [
                History::Open,
                History::Check,
                History::Bet,
                History::CheckBet,
            ] {
                for edge in CfrInfo::choices(&view(rank, node)) {
                    eprintln!(
                        "  {} {:>10} {:>6} {:>8.4}",
                        rank,
                        format!("{:?}", node),
                        edge,
                        subpolicy(&profile, rank, node, edge),
                    );
                }
            }
        }
        eprintln!("\n=== structural checks ===");
        assert!(regret < 0.01, "regret {regret:.6} >= 0.01");
        assert!(
            subpolicy(&profile, Rank::K, History::CheckBet, KuhnEdge::Call) > 0.90,
            "K|XB call"
        );
        eprintln!("  regret < 0.01 ✓");
        eprintln!("  K|XB calls ✓");
    }
}
