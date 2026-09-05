//! Tier 3: what a logged decision was worth, against what ours was worth.
//!
//! Tier 2 ([`crate::Agreement`]) can only say whether we *agreed* with the log.
//! Agreement is not strength: the log records what happened after Pluribus's
//! action, so the moment our bot deviates there is no continuation to score.
//!
//! The way out is to stop looking for the continuation and generate it. At each
//! logged decision, play the hand out twice with our own strategy driving **both**
//! seats — once behind the action the log took, once behind the action we would
//! have taken — and difference the two:
//!
//! ```text
//!               ┌── log's action ──▶ our strategy plays it out ──▶ chips
//!   decision ───┤
//!               └── our action  ───▶ our strategy plays it out ──▶ chips
//! ```
//!
//! [`Duel::delta`] is the difference, in big blinds: **how much substituting the
//! logged decision for ours gains against our own strategy.** Both branches run
//! through the identical rollout, so whatever our value function gets wrong
//! largely cancels out of the difference.
//!
//! # What the sign actually measures
//!
//! δ is not a comparison of two players. Both continuations are played by *our*
//! strategy, so δ asks one question about one strategy: **is there a better move
//! here than the one we make, against ourselves?**
//!
//! A converged blueprint is best-responding to itself. Every action in its
//! support has equal value against it and every action outside is worse, so
//! substituting any other action can only lose:
//!
//! ```text
//!   self-consistent strategy  ⟹  δ ≤ 0 for every substitution
//! ```
//!
//! So **δ > 0 is a positive measurement that we are not converged at that node —
//! the log found a profitable deviation against us.** That makes [`Ledger`] a
//! local best response with the log's real decisions as the probe set instead of
//! an exhaustive search, and [`Ledger::delta`] **a lower bound on our own
//! exploitability**, in big blinds per decision. Lower is better; zero is the
//! target; the bound is only ever loose in the safe direction, since a probe set
//! of a few thousand human-chosen actions cannot find every leak.
//!
//! It needs a scale to be legible, and the cheapest one is a strategy that is
//! obviously bad. `cargo run -p phh --example rollout` duels the corpus against
//! one that always checks or calls: **+0.6188 bb/decision**. That is what "very
//! exploitable" reads as on this probe set.
//!
//! Read [`Ledger::t`] before any of it: a sign inside two standard errors is
//! noise. And note the residual bias runs in our favor — the log's action was
//! chosen against five human professionals rather than against us, and our bot
//! plays the continuation — so δ ≈ 0 is weaker evidence of soundness than δ > 0
//! is of a leak.
//!
//! # Where the cards come from
//!
//! The PHH corpus deals every player's hole cards face-up in the log — all six,
//! in all 10,000 hands — and records however much of the board the dealer ran
//! out before the hand ended. So a [`World`] samples *only* the streets the log
//! never reached. Using the real cards is not cheating: no strategy in the
//! rollout ever sees them, they enter only at settlement. Holding them fixed
//! across both branches is the common-random-numbers trick, and it is what makes
//! the difference of two noisy means far tighter than either mean.

use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;
use rand::SeedableRng;

/// A rollout that fails to advance this many times has hit a state the engine
/// and the grid disagree about. Well above the deepest line our tree can hold,
/// so tripping it is a bug rather than a long hand.
const STEPS: usize = 64;

/// The strategy's chosen action when it has never seen the information set.
///
/// Checking or calling is the least opinionated thing to do with no opinion, and
/// [`Wager::misses`] counts every time it fires so the fallback can never quietly
/// carry the result.
fn shrug(game: &Game) -> Action {
    game.passive()
}

/// Chips as big blinds, the unit every value in this module is quoted in.
fn bb(chips: Chips) -> Utility {
    Utility::from(chips) / Utility::from(B_BLIND)
}

/// A complete runout: the five board cards a hand would see.
///
/// Everything the log dealt is kept and only the streets it never reached are
/// sampled, so a hand that went to showdown contributes its real board and a
/// hand that ended preflop contributes a sampled one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World(Vec<Card>);

impl World {
    /// The five cards, in deal order.
    pub fn board(&self) -> &[Card] {
        &self.0
    }

    /// The cards a deal onto `street` turns over.
    fn deal(&self, street: Street) -> Option<Action> {
        self.0
            .get(street.n_board() - street.n_revealed()..street.n_board())
            .map(|cards| Action::Draw(Hand::from(cards.to_vec())))
    }
}

/// Board cards on a complete runout.
fn full() -> usize {
    Street::Rive.n_board()
}

/// Draws `n` runouts consistent with what `spot` had seen, completing the log's
/// board from the undealt remainder of the deck.
///
/// Deterministic in `seed`: the same spot and seed always give the same worlds,
/// so a rerun reproduces a number and — more importantly — the two branches of a
/// [`Duel`] are played in identical cards. When the log already ran the board
/// out there is nothing left to sample, and a single world is returned however
/// large `n` is.
pub fn worlds(spot: &Spot, n: usize, seed: u64) -> Vec<World> {
    let known = spot.board().to_vec();
    if known.len() >= full() {
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
                        rand::seq::index::sample(rng, live.len(), full() - known.len())
                            .into_iter()
                            .map(|i| live[i]),
                    )
                    .collect(),
            )
        })
        .collect()
}

/// What a line was worth at one spot, averaged over a shared set of worlds.
///
/// A single action's wager weights every world equally. A whole strategy's
/// weights each of its actions by the mass it puts there, which is why the
/// average is carried as a weighted sum rather than a count.
#[derive(Debug, Clone, Copy, Default)]
pub struct Wager {
    total: f64,
    weight: f64,
    worlds: usize,
    misses: usize,
}

impl Wager {
    /// How many rollouts went into the mean.
    pub fn worlds(&self) -> usize {
        self.worlds
    }

    /// Decisions inside the rollout where the strategy had no distribution and
    /// the check-or-call fallback answered for it.
    pub fn misses(&self) -> usize {
        self.misses
    }

    /// Mean big blinds won over the whole hand.
    ///
    /// Absolute, so it carries the chips committed before the decision — which
    /// are identical in both branches of a [`Duel`] and cancel out of
    /// [`Duel::delta`]. Only the difference is interpretable.
    pub fn value(&self) -> Utility {
        match self.weight {
            weight if weight > 0.0 => (self.total / weight) as Utility,
            _ => 0.0,
        }
    }
}

impl std::ops::Add for Wager {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            total: self.total + other.total,
            weight: self.weight + other.weight,
            worlds: self.worlds + other.worlds,
            misses: self.misses + other.misses,
        }
    }
}

impl std::iter::Sum for Wager {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::default(), std::ops::Add::add)
    }
}

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
        self.theirs.value() - self.ours.value()
    }
}

/// Scores one logged decision against `oracle`, over `worlds`.
///
/// Our side is the policy's **expectation**, not one draw from it:
///
/// ```text
///   δ = EV(played) − Σₐ p(a)·EV(a)  =  Σₐ p(a)·[ EV(played) − EV(a) ]
/// ```
///
/// The term where `a` is what they played is exactly zero — same action, same
/// worlds, same random stream — and every other term is a difference of two
/// rollouts through identical cards. Drawing one action instead would be
/// unbiased and far noisier, because most of the per-decision variance lives in
/// the mixture rather than in the runout: agreement is barely better than a coin
/// flip, so half the spots would contribute the full spread between two entirely
/// different lines through a poker hand.
///
/// It costs a rollout per legal action instead of one. Worth it — the same wall
/// clock spent on more worlds buys much less.
pub fn duel<O>(oracle: &O, spot: &Spot, worlds: &[World], seed: u64) -> Option<Duel>
where
    O: Oracle,
{
    let policy = oracle.policy(spot.witness());
    let game = spot.game();
    Some(Duel {
        street: spot.street(),
        played: spot.edge(),
        chosen: policy.favorite()?,
        theirs: wager(oracle, spot, worlds, seed, spot.played(), 1.0),
        ours: policy
            .edges()
            .map(|edge| wager(oracle, spot, worlds, seed, game.snap(game.actionize(edge)), policy.mass(edge)))
            .sum(),
    })
}

/// Plays `spot` out once per world, opening with `open`, weighted by `mass`.
///
/// Every rollout of a spot runs on a stream keyed only by `(seed, world)` — never
/// by which action opened it — so two branches that reach the same state see the
/// same cards *and* the same choices, and cancel to the bit rather than to
/// something merely small.
fn wager<O>(oracle: &O, spot: &Spot, worlds: &[World], seed: u64, open: Action, mass: Probability) -> Wager
where
    O: Oracle,
{
    worlds
        .iter()
        .enumerate()
        .filter_map(|(i, world)| Playout::new(spot, world).open(open)?.run(oracle, &mut stream(seed, i)))
        .fold(Wager::default(), |mut wager, (value, misses)| {
            wager.total += f64::from(mass) * f64::from(value);
            wager.weight += f64::from(mass);
            wager.worlds += 1;
            wager.misses += misses;
            wager
        })
}

/// The random stream for one world of one spot. Deterministic in both, and
/// deliberately blind to which action is being scored.
fn stream(seed: u64, world: usize) -> rand::rngs::SmallRng {
    rand::rngs::SmallRng::seed_from_u64(seed ^ world as u64)
}

/// One rollout in progress.
///
/// Carries the action prefix alongside the engine state because a [`Witness`] —
/// the only thing an [`Oracle`] can be asked about — has to be rebuilt from
/// scratch at every decision.
struct Playout<'a> {
    spot: &'a Spot,
    world: &'a World,
    game: Game,
    actions: Vec<Action>,
    misses: usize,
}

impl<'a> Playout<'a> {
    fn new(spot: &'a Spot, world: &'a World) -> Self {
        Self {
            spot,
            world,
            game: spot.game(),
            actions: spot.actions().to_vec(),
            misses: 0,
        }
    }

    /// Takes the branch's opening action, the one thing the two branches differ on.
    fn open(mut self, action: Action) -> Option<Self> {
        self.apply(action)?;
        Some(self)
    }

    /// Big blinds won by the hero, and how often the strategy had nothing to say.
    fn run<O>(mut self, oracle: &O, rng: &mut rand::rngs::SmallRng) -> Option<(Utility, usize)>
    where
        O: Oracle,
    {
        for _ in 0..STEPS {
            let action = match self.game.turn() {
                Turn::Terminal => return Some((self.value(), self.misses)),
                Turn::Chance => self.world.deal(self.game.street().next())?,
                Turn::Choice(seat) => self.decide(oracle, seat, rng).unwrap_or_else(|| {
                    self.misses += 1;
                    shrug(&self.game)
                }),
            };
            self.apply(action)?;
        }
        None
    }

    /// Asks the strategy what the seat to act should do, and translates it into
    /// chips. `None` when the strategy has never seen the information set.
    fn decide<O>(&self, oracle: &O, seat: usize, rng: &mut rand::rngs::SmallRng) -> Option<Action>
    where
        O: Oracle,
    {
        oracle
            .policy(&self.witness(seat)?)
            .draw(rng)
            .map(|edge| self.game.snap(self.game.actionize(edge)))
    }

    /// What `seat` has seen — its own hole cards, the board so far, and every
    /// decision either player has made.
    fn witness(&self, seat: usize) -> Option<Witness> {
        Witness::try_arrange_with(
            Turn::Choice(seat),
            Arrangement::from(
                Vec::<Card>::from(Hand::from(*self.spot.holes().get(seat)?))
                    .into_iter()
                    .chain(self.world.board().iter().copied().take(self.game.street().n_board()))
                    .collect::<Vec<Card>>(),
            ),
            self.spot.stacks(),
            self.actions.iter().copied().filter(Action::is_choice).collect(),
        )
        .ok()
    }

    fn apply(&mut self, action: Action) -> Option<()> {
        self.game = self.game.try_apply(action).ok()?;
        self.actions.push(action);
        Some(())
    }

    fn value(&self) -> Utility {
        bb(self
            .game
            .settlements()
            .get(self.spot.witness().turn().position())
            .map_or(0, Settlement::won))
    }
}

/// A scored set of duels — the Tier 3 counterpart to [`crate::Agreement`].
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    duels: Vec<Duel>,
}

impl FromIterator<Duel> for Ledger {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Duel>,
    {
        Self {
            duels: iter.into_iter().collect(),
        }
    }
}

impl Ledger {
    pub fn duels(&self) -> &[Duel] {
        &self.duels
    }

    pub fn n(&self) -> usize {
        self.duels.len()
    }

    pub fn on(&self, street: Street) -> Self {
        self.duels.iter().copied().filter(|d| d.street == street).collect()
    }

    /// Mean big blinds the logged decisions gained over ours.
    pub fn delta(&self) -> Utility {
        self.mean(Duel::delta) as Utility
    }

    /// Population standard deviation of the per-decision gain, in big blinds.
    pub fn spread(&self) -> Utility {
        let mean = self.mean(Duel::delta);
        self.mean(|d| (f64::from(d.delta()) - mean).powi(2) as Utility).sqrt() as Utility
    }

    /// [`delta`](Self::delta) per hundred decisions — the headline. Not bb/100
    /// hands: these are decisions, and a hand holds several.
    pub fn per_hundred(&self) -> Utility {
        100.0 * self.delta()
    }

    /// Standard error of [`delta`](Self::delta), in big blinds.
    pub fn stderr(&self) -> Utility {
        match self.n() {
            0 | 1 => 0.0,
            n => self.spread() / ((n - 1) as Utility).sqrt(),
        }
    }

    /// [`delta`](Self::delta) in standard errors. Inside ±2 the sign is noise.
    pub fn t(&self) -> f64 {
        match self.stderr() {
            0.0 => 0.0,
            e => f64::from(self.delta() / e),
        }
    }

    /// How many worlds every decision was averaged over, in total.
    pub fn worlds(&self) -> usize {
        self.duels.iter().map(|d| d.theirs.worlds + d.ours.worlds).sum()
    }

    /// Rollout decisions where the strategy had no distribution at all.
    pub fn misses(&self) -> usize {
        self.duels.iter().map(|d| d.theirs.misses + d.ours.misses).sum()
    }

    /// Per-edge: how often the log played it, and what playing it gained. Where
    /// the strategy is actually losing chips, rather than merely disagreeing.
    pub fn by_edge(&self) -> Vec<(Edge, usize, Utility)> {
        let mut edges = self.duels.iter().map(|d| d.played).collect::<Vec<_>>();
        edges.sort_unstable();
        edges.dedup();
        let mut rows = edges
            .into_iter()
            .map(|edge| {
                let on = self
                    .duels
                    .iter()
                    .copied()
                    .filter(|d| d.played == edge)
                    .collect::<Self>();
                (edge, on.n(), on.delta())
            })
            .collect::<Vec<_>>();
        rows.sort_by_key(|(_, n, _)| std::cmp::Reverse(*n));
        rows
    }

    /// Population mean of a per-duel quantity, accumulated wide so a few
    /// thousand `f32` samples do not drift.
    fn mean(&self, value: impl Fn(&Duel) -> Utility) -> f64 {
        match self.duels.len() {
            0 => 0.0,
            n => self.duels.iter().map(|d| f64::from(value(d))).sum::<f64>() / n as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A strategy that always checks or calls: enough to drive a rollout to a
    /// terminal node without a blueprint.
    struct Passive;

    impl Oracle for Passive {
        fn policy(&self, witness: &Witness) -> Policy {
            let game = witness.head();
            [(if game.may_check() { Edge::Check } else { Edge::Call }, 1.0)]
                .into_iter()
                .collect()
        }
    }

    /// A strategy that says nothing, ever — every decision is a miss.
    struct Mute;

    impl Oracle for Mute {
        fn policy(&self, _: &Witness) -> Policy {
            Policy::default()
        }
    }

    /// A strategy that genuinely mixes, so its continuation depends on the
    /// random stream. Needed to see stream desync at all — a pure strategy
    /// returns the same line whatever the rng says.
    struct Mixed;

    impl Oracle for Mixed {
        fn policy(&self, witness: &Witness) -> Policy {
            let game = witness.head();
            [
                (if game.may_check() { Edge::Check } else { Edge::Call }, 2.0),
                (if game.may_check() { Edge::Raise(Odds::new(1, 2)) } else { Edge::Fold }, 1.0),
            ]
            .into_iter()
            .collect()
        }
    }

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    fn spots(actions: &str) -> Vec<Spot> {
        let text = format!(
            "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
             blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
             starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
             actions = [{DEAL}, {actions}]\n"
        );
        let record = crate::parse(&text, "test").unwrap();
        let outcome = crate::replay(&record).unwrap();
        crate::spots(&record, &outcome).expect("reduces")
    }

    /// Button opens, big blind calls, both check the flop, big blind bets the
    /// turn and the button folds. The river is never dealt.
    fn worked() -> Vec<Spot> {
        spots(
            "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
             'd db 7d5h9d', 'p2 cc', 'p6 cc', 'd db 2s', 'p2 cbr 250', 'p6 f'",
        )
    }

    #[test]
    fn a_world_completes_the_board_the_log_left_short() {
        let spot = &worked()[2];
        assert_eq!(spot.board().len(), 4, "the log stopped on the turn");
        for world in worlds(spot, 8, 1) {
            assert_eq!(world.board().len(), full());
            assert_eq!(&world.board()[..4], spot.board(), "the logged cards are kept");
        }
    }

    /// A sampled river must not be a card somebody is already holding.
    #[test]
    fn sampled_cards_never_collide_with_the_deal() {
        let spot = &worked()[2];
        let dead = spot
            .holes()
            .iter()
            .flat_map(|h| Vec::<Card>::from(Hand::from(*h)))
            .chain(spot.board().iter().copied())
            .collect::<Vec<_>>();
        for world in worlds(spot, 64, 7) {
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
        let spot = &worked()[2];
        assert_eq!(worlds(spot, 8, 3), worlds(spot, 8, 3));
        assert_ne!(worlds(spot, 8, 3), worlds(spot, 8, 4));
    }

    /// The board was run out in full, so there is nothing left to sample and one
    /// world says everything sixty-four would.
    #[test]
    fn a_finished_board_collapses_to_a_single_world() {
        let spots = spots(
            "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
             'd db 7d5h9d', 'p2 cc', 'p6 cc', 'd db 2s', 'p2 cc', 'p6 cc', \
             'd db Kh', 'p2 cc', 'p6 cc'",
        );
        let spot = spots.last().unwrap();
        assert_eq!(spot.board().len(), full());
        assert_eq!(worlds(spot, 64, 0).len(), 1);
    }

    /// Every rollout has to reach a terminal node and settle, or the value is a
    /// silent zero.
    #[test]
    fn a_rollout_reaches_a_showdown() {
        let spot = &worked()[0];
        let ref worlds = worlds(spot, 4, 11);
        let duel = duel(&Passive, spot, worlds, 11).expect("scored");
        assert_eq!(duel.theirs().worlds(), worlds.len());
        assert_eq!(duel.ours().worlds(), worlds.len());
        assert_eq!(duel.theirs().misses(), 0);
    }

    /// Both branches open with the same action here, so common random numbers
    /// have to drive the difference to exactly zero — not merely close to it.
    #[test]
    fn identical_branches_cancel_exactly() {
        let spot = &worked()[0];
        assert_eq!(spot.edge(), Edge::Check, "the log checked");
        let duel = duel(&Passive, spot, &worlds(spot, 16, 5), 5).expect("scored");
        assert_eq!(duel.chosen(), Edge::Check, "and so does a passive strategy");
        assert_eq!(duel.delta(), 0.0);
    }

    /// Our side is a mixture over every legal action, so it costs one rollout
    /// per action rather than one per world.
    #[test]
    fn our_side_scores_every_action_not_a_sample() {
        let spot = &worked()[0];
        let ref worlds = worlds(spot, 4, 5);
        let duel = duel(&Mixed, spot, worlds, 5).expect("scored");
        assert_eq!(duel.theirs().worlds(), worlds.len());
        assert_eq!(duel.ours().worlds(), 2 * worlds.len(), "Mixed offers two edges");
    }

    /// The term where the mixture's action *is* what they played has to vanish
    /// exactly. Same action, same worlds, same random stream — so if the streams
    /// were ever keyed on which action opened the rollout, this would drift and
    /// the whole variance reduction would quietly stop working.
    #[test]
    fn the_matching_term_of_the_mixture_cancels_to_the_bit() {
        let spot = &worked()[0];
        let ref worlds = worlds(spot, 16, 5);
        assert_eq!(spot.edge(), Edge::Check, "the log checked");
        let theirs = wager(&Mixed, spot, worlds, 5, spot.played(), 1.0);
        let mirror = wager(&Mixed, spot, worlds, 5, spot.played(), 0.25);
        assert_eq!(theirs.value(), mirror.value(), "weight must not move the mean");
        assert_eq!(duel(&Mixed, spot, worlds, 5).expect("scored").theirs().value(), theirs.value());
    }

    /// A strategy that never deviates has nothing to gain from deviating.
    #[test]
    fn identical_strategies_cancel_exactly() {
        let spot = &worked()[0];
        let duel = duel(&Passive, spot, &worlds(spot, 16, 5), 5).expect("scored");
        assert_eq!(duel.delta(), 0.0);
    }

    /// A strategy with nothing to say still produces a number, and says so.
    #[test]
    fn a_silent_strategy_is_counted_not_hidden() {
        let spot = &worked()[0];
        assert!(duel(&Mute, spot, &worlds(spot, 2, 0), 0).is_none(), "no chosen edge");
        let wager = wager(&Mute, spot, &worlds(spot, 2, 0), 0, spot.played(), 1.0);
        assert_eq!(wager.worlds(), 2);
        assert!(wager.misses() > 0);
    }

    fn synthetic(delta: Utility) -> Duel {
        Duel {
            street: Street::Flop,
            played: Edge::Check,
            chosen: Edge::Fold,
            theirs: Wager {
                total: f64::from(delta),
                weight: 1.0,
                worlds: 1,
                misses: 0,
            },
            ours: Wager::default(),
        }
    }

    #[test]
    fn the_ledger_averages_and_scales() {
        let ledger = [1.0, 2.0, 3.0, 4.0].map(synthetic).into_iter().collect::<Ledger>();
        assert_eq!(ledger.n(), 4);
        assert_eq!(ledger.delta(), 2.5);
        assert_eq!(ledger.per_hundred(), 250.0);
    }

    /// A constant gain has no spread, so `t` cannot be computed from it; a noisy
    /// one straddling zero has to land well inside the believable range.
    #[test]
    fn t_separates_a_signal_from_noise() {
        let steady = (0..64).map(|_| synthetic(1.0)).collect::<Ledger>();
        let noisy = (0..64)
            .map(|i| synthetic(if i % 2 == 0 { 1.0 } else { -1.0 }))
            .collect::<Ledger>();
        assert_eq!(steady.stderr(), 0.0);
        assert_eq!(noisy.delta(), 0.0);
        assert!(noisy.t().abs() < 2.0);
    }
}
