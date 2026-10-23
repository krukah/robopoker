#![doc = include_str!("../doc/duel.md")]

use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;
use rand::SeedableRng;

/// A rollout that fails to advance this many times has hit a state the engine
/// and the grid disagree about. Well above the deepest line our tree can hold,
/// so tripping it is a bug rather than a long hand.
pub(crate) const STEPS: usize = 64;

/// One logged decision, scored both ways over the same worlds.
#[derive(Debug, Clone, Copy)]
pub struct Duel {
    street: Street,
    played: Edge,
    chosen: Edge,
    theirs: Wager,
    ours: Wager,
}

impl Duel {
    pub fn street(&self) -> Street {
        self.street
    }

    /// The edge the logged action snapped to.
    pub fn played(&self) -> Edge {
        self.played
    }

    /// The edge our strategy leans on hardest here. Reporting only — the rollout
    /// draws from the whole distribution rather than playing this purely.
    pub fn chosen(&self) -> Edge {
        self.chosen
    }

    /// What the logged action was worth.
    pub fn theirs(&self) -> &Wager {
        &self.theirs
    }

    /// What our own action was worth.
    pub fn ours(&self) -> &Wager {
        &self.ours
    }

    /// Big blinds the logged decision gains over ours. Positive means the log
    /// out-decided us here.
    pub fn delta(&self) -> Utility {
        self.theirs().value() - self.ours().value()
    }
}

impl Duel {
    /// The strategy's chosen action when it has never seen the information set.
    ///
    /// Checking or calling is the least opinionated thing to do with no opinion, and
    /// [`Wager::misses`] counts every time it fires so the fallback can never quietly
    /// carry the result.
    pub(crate) fn shrug(game: &Game) -> Action {
        game.passive()
    }

    /// Chips as big blinds, the unit every value in this module is quoted in.
    pub(crate) fn bb(chips: Chips) -> Utility {
        Utility::from(chips) / Utility::from(B_BLIND)
    }

    /// Board cards on a complete runout.
    fn full() -> usize {
        Street::Rive.n_board()
    }

    /// Draws `n` runouts consistent with what `spot` had seen, completing the log's board from the
    /// undealt remainder of the deck.
    ///
    /// Deterministic in `seed`: the same spot and seed always give the same worlds, so a rerun
    /// reproduces a number and — more importantly — the two branches of a [`Duel`] are played in
    /// identical cards.
    pub fn worlds(spot: &Spot, n: usize, seed: u64) -> Vec<World> {
        let known = spot.board().to_vec();
        if known.len() >= Duel::full() {
            return vec![World(known)];
        }
        let ref mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
        let dead = spot
            .holes()
            .iter()
            .flat_map(|hole| Vec::<Card>::from(Hand::from(*hole)))
            .chain(known.iter().copied())
            .fold(Hand::empty(), |hand, card| Hand::add(hand, Hand::from(card)));
        let live = Vec::<Card>::from(dead.complement());
        (0..n.max(1))
            .map(|_| {
                World(
                    known
                        .iter()
                        .copied()
                        .chain(
                            rand::seq::index::sample(rng, live.len(), Duel::full() - known.len())
                                .into_iter()
                                .map(|i| live[i]),
                        )
                        .collect(),
                )
            })
            .collect()
    }

    /// Scores one logged decision against `oracle`, over `worlds`.
    ///
    /// The term where `a` is what they played is exactly zero — same action, same worlds, same
    /// random stream — and every other term is a difference of two rollouts through identical
    /// cards.
    pub fn play<O>(oracle: &O, spot: &Spot, worlds: &[World], seed: u64) -> Option<Duel>
    where
        O: Oracle,
    {
        let policy = oracle.policy(spot.witness());
        let game = spot.game();
        Some(Duel {
            street: spot.street(),
            played: spot.edge(),
            chosen: policy.favorite()?,
            theirs: Duel::wager(oracle, spot, worlds, seed, spot.played(), 1.0),
            ours: policy
                .edges()
                .map(|edge| Duel::wager(oracle, spot, worlds, seed, game.snap(game.actionize(edge)), policy.mass(edge)))
                .sum(),
        })
    }

    /// Plays `spot` out once per world, opening with `open`, weighted by `mass`.
    ///
    /// Every rollout of a spot runs on a stream keyed only by `(seed, world)` — never
    /// by which action opened it — so two branches that reach the same state see the
    /// same cards *and* the same choices, and cancel to the bit rather than to
    /// something merely small.
    pub(crate) fn wager<O>(
        oracle: &O,
        spot: &Spot,
        worlds: &[World],
        seed: u64,
        open: Action,
        mass: Probability,
    ) -> Wager
    where
        O: Oracle,
    {
        worlds
            .iter()
            .enumerate()
            .filter_map(|(i, world)| {
                Playout::new(spot, world)
                    .open(open)?
                    .run(oracle, &mut Duel::stream(seed, i))
            })
            .fold(Wager::default(), |mut wager, (value, misses)| {
                *wager.total_mut() += f64::from(mass) * f64::from(value);
                *wager.weight_mut() += f64::from(mass);
                *wager.worlds_mut() += 1;
                *wager.misses_mut() += misses;
                wager
            })
    }

    /// The random stream for one world of one spot. Deterministic in both, and
    /// deliberately blind to which action is being scored.
    fn stream(seed: u64, world: usize) -> rand::rngs::SmallRng {
        rand::rngs::SmallRng::seed_from_u64(seed ^ world as u64)
    }
}

impl Duel {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::*;

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    impl Duel {
        fn spots(actions: &str) -> Vec<Spot> {
            let text = format!(
                "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
                 blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
                 starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
                 actions = [{DEAL}, {actions}]\n"
            );
            let record = crate::Parse::parse(&text, "test").unwrap();
            crate::Spot::spots(&record, &crate::Replay::replay(&record).unwrap()).expect("reduces")
        }

        /// Button opens, big blind calls, both check the flop, big blind bets the
        /// turn and the button folds. The river is never dealt.
        fn worked() -> Vec<Spot> {
            Self::spots(
                "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
                 'd db 7d5h9d', 'p2 cc', 'p6 cc', 'd db 2s', 'p2 cbr 250', 'p6 f'",
            )
        }

        fn synthetic(delta: Utility) -> Self {
            Self {
                street: Street::Flop,
                played: Edge::Check,
                chosen: Edge::Fold,
                theirs: Wager::new(f64::from(delta), 1.0, 1, 0),
                ours: Wager::default(),
            }
        }
    }

    #[test]
    fn a_world_completes_the_board_the_log_left_short() {
        let spot = &Duel::worked()[2];
        assert_eq!(spot.board().len(), 4, "the log stopped on the turn");
        for world in Duel::worlds(spot, 8, 1) {
            assert_eq!(world.board().len(), Duel::full());
            assert_eq!(&world.board()[..4], spot.board(), "the logged cards are kept");
        }
    }

    /// A sampled river must not be a card somebody is already holding.
    #[test]
    fn sampled_cards_never_collide_with_the_deal() {
        let spot = &Duel::worked()[2];
        let dead = spot
            .holes()
            .iter()
            .flat_map(|h| Vec::<Card>::from(Hand::from(*h)))
            .chain(spot.board().iter().copied())
            .collect::<Vec<_>>();
        for world in Duel::worlds(spot, 64, 7) {
            for card in world.board() {
                assert!(
                    dead.iter().filter(|c| *c == card).count() <= usize::from(spot.board().contains(card)),
                    "{card} was already in play",
                );
            }
        }
    }

    #[test]
    fn worlds_are_deterministic_in_the_seed() {
        let spot = &Duel::worked()[2];
        assert_eq!(Duel::worlds(spot, 8, 3), Duel::worlds(spot, 8, 3));
        assert_ne!(Duel::worlds(spot, 8, 3), Duel::worlds(spot, 8, 4));
    }

    /// The board was run out in full, so there is nothing left to sample and one
    /// world says everything sixty-four would.
    #[test]
    fn a_finished_board_collapses_to_a_single_world() {
        let spots = Duel::spots(
            "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
             'd db 7d5h9d', 'p2 cc', 'p6 cc', 'd db 2s', 'p2 cc', 'p6 cc', \
             'd db Kh', 'p2 cc', 'p6 cc'",
        );
        let spot = spots.last().unwrap();
        assert_eq!(spot.board().len(), Duel::full());
        assert_eq!(Duel::worlds(spot, 64, 0).len(), 1);
    }

    /// Every rollout has to reach a terminal node and settle, or the value is a
    /// silent zero.
    #[test]
    fn a_rollout_reaches_a_showdown() {
        let spot = &Duel::worked()[0];
        let ref worlds = Duel::worlds(spot, 4, 11);
        let duel = Duel::play(&Passive, spot, worlds, 11).expect("scored");
        assert_eq!(duel.theirs().worlds(), worlds.len());
        assert_eq!(duel.ours().worlds(), worlds.len());
        assert_eq!(duel.theirs().misses(), 0);
    }

    /// Both branches open with the same action here, so common random numbers
    /// have to drive the difference to exactly zero — not merely close to it.
    #[test]
    fn identical_branches_cancel_exactly() {
        let spot = &Duel::worked()[0];
        assert_eq!(spot.edge(), Edge::Check, "the log checked");
        let duel = Duel::play(&Passive, spot, &Duel::worlds(spot, 16, 5), 5).expect("scored");
        assert_eq!(duel.chosen(), Edge::Check, "and so does a passive strategy");
        assert_eq!(duel.delta(), 0.0);
    }

    /// Our side is a mixture over every legal action, so it costs one rollout
    /// per action rather than one per world.
    #[test]
    fn our_side_scores_every_action_not_a_sample() {
        let spot = &Duel::worked()[0];
        let ref worlds = Duel::worlds(spot, 4, 5);
        let duel = Duel::play(&Mixed, spot, worlds, 5).expect("scored");
        assert_eq!(duel.theirs().worlds(), worlds.len());
        assert_eq!(duel.ours().worlds(), 2 * worlds.len(), "Mixed offers two edges");
    }

    /// The term where the mixture's action *is* what they played has to vanish exactly.
    #[test]
    fn the_matching_term_of_the_mixture_cancels_to_the_bit() {
        let spot = &Duel::worked()[0];
        let ref worlds = Duel::worlds(spot, 16, 5);
        assert_eq!(spot.edge(), Edge::Check, "the log checked");
        let theirs = Duel::wager(&Mixed, spot, worlds, 5, spot.played(), 1.0);
        let mirror = Duel::wager(&Mixed, spot, worlds, 5, spot.played(), 0.25);
        assert_eq!(theirs.value(), mirror.value(), "weight must not move the mean");
        assert_eq!(Duel::play(&Mixed, spot, worlds, 5).expect("scored").theirs().value(), theirs.value());
    }

    /// A strategy that never deviates has nothing to gain from deviating.
    #[test]
    fn identical_strategies_cancel_exactly() {
        let spot = &Duel::worked()[0];
        let duel = Duel::play(&Passive, spot, &Duel::worlds(spot, 16, 5), 5).expect("scored");
        assert_eq!(duel.delta(), 0.0);
    }

    /// A strategy with nothing to say still produces a number, and says so.
    #[test]
    fn a_silent_strategy_is_counted_not_hidden() {
        let spot = &Duel::worked()[0];
        assert!(Duel::play(&Mute, spot, &Duel::worlds(spot, 2, 0), 0).is_none(), "no chosen edge");
        let wager = Duel::wager(&Mute, spot, &Duel::worlds(spot, 2, 0), 0, spot.played(), 1.0);
        assert_eq!(wager.worlds(), 2);
        assert!(wager.misses() > 0);
    }

    #[test]
    fn the_ledger_averages_and_scales() {
        let ledger = [1.0, 2.0, 3.0, 4.0]
            .map(Duel::synthetic)
            .into_iter()
            .collect::<Ledger>();
        assert_eq!(ledger.n(), 4);
        assert_eq!(ledger.delta(), 2.5);
        assert_eq!(ledger.per_hundred(), 250.0);
    }

    /// A constant gain has no spread, so `t` cannot be computed from it; a noisy
    /// one straddling zero has to land well inside the believable range.
    #[test]
    fn t_separates_a_signal_from_noise() {
        let steady = (0..64).map(|_| Duel::synthetic(1.0)).collect::<Ledger>();
        let noisy = (0..64)
            .map(|i| Duel::synthetic(if i % 2 == 0 { 1.0 } else { -1.0 }))
            .collect::<Ledger>();
        assert_eq!(steady.stderr(), 0.0);
        assert_eq!(noisy.delta(), 0.0);
        assert!(noisy.t().abs() < 2.0);
    }
}
