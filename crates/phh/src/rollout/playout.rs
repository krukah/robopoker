use super::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// One rollout in progress.
///
/// Carries the action prefix alongside the engine state because a [`Witness`] —
/// the only thing an [`Oracle`] can be asked about — has to be rebuilt from
/// scratch at every decision.
pub(crate) struct Playout<'spot> {
    spot: &'spot Spot,
    world: &'spot World,
    game: Game,
    actions: Vec<Action>,
    misses: usize,
}
impl<'spot> Playout<'spot> {
    pub(crate) fn new(spot: &'spot Spot, world: &'spot World) -> Self {
        Self {
            spot,
            world,
            game: spot.game(),
            actions: spot.actions().to_vec(),
            misses: 0,
        }
    }

    fn spot(&self) -> &'spot Spot {
        self.spot
    }

    fn world(&self) -> &'spot World {
        self.world
    }

    fn game(&self) -> Game {
        self.game
    }

    fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    fn actions(&self) -> &[Action] {
        &self.actions
    }

    fn actions_mut(&mut self) -> &mut Vec<Action> {
        &mut self.actions
    }

    fn misses(&self) -> usize {
        self.misses
    }

    fn misses_mut(&mut self) -> &mut usize {
        &mut self.misses
    }

    /// Takes the branch's opening action, the one thing the two branches differ on.
    pub(crate) fn open(mut self, action: Action) -> Option<Self> {
        self.apply(action)?;
        Some(self)
    }

    /// Big blinds won by the hero, and how often the strategy had nothing to say.
    pub(crate) fn run<O>(mut self, oracle: &O, rng: &mut rand::rngs::SmallRng) -> Option<(Utility, usize)>
    where
        O: Oracle,
    {
        for _ in 0..STEPS {
            let action = match self.game().turn() {
                Turn::Terminal => return Some((self.value(), self.misses())),
                Turn::Chance => self.world().deal(self.game().street().next())?,
                Turn::Choice(seat) => self.decide(oracle, seat, rng).unwrap_or_else(|| {
                    *self.misses_mut() += 1;
                    Duel::shrug(&self.game())
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
            .map(|edge| self.game().snap(self.game().actionize(edge)))
    }

    /// What `seat` has seen — its own hole cards, the board so far, and every
    /// decision either player has made.
    fn witness(&self, seat: usize) -> Option<Witness> {
        Witness::try_arrange_with(
            Turn::Choice(seat),
            Arrangement::from(
                Vec::<Card>::from(Hand::from(*self.spot().holes().get(seat)?))
                    .into_iter()
                    .chain(
                        self.world()
                            .board()
                            .iter()
                            .copied()
                            .take(self.game().street().n_board()),
                    )
                    .collect::<Vec<Card>>(),
            ),
            self.spot().stacks(),
            self.actions().iter().copied().filter(Action::is_choice).collect(),
        )
        .ok()
    }

    fn apply(&mut self, action: Action) -> Option<()> {
        *self.game_mut() = self.game().try_apply(action).ok()?;
        self.actions_mut().push(action);
        Some(())
    }

    fn value(&self) -> Utility {
        Duel::bb(
            self.game()
                .settlements()
                .get(self.spot().witness().turn().position())
                .map_or(0, Settlement::won),
        )
    }
}
