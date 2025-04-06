use crate::cards::card::Card;
use crate::cards::hole::Hole;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::gameplay::action::Action;
use crate::gameplay::game::Game;
use crate::gameplay::path::Path;
use crate::gameplay::turn::Turn;

/// a complete representation of perfect recall game history
/// from the perspective of the hero. intended use is for
/// the path to be populated only with choice actions,
/// since we internally keep track of chance actions
/// by conditioning on the observed cards.
///
/// note that this struct implicitly assumes:
/// - default stacks
/// - default blinds
/// - default dealer position
/// - unordered private and community cards
#[derive(Debug, Clone)]
pub struct Recall {
    hero: Turn,
    seen: Observation, // could be replaced by Hole + Board + BetHistory(Vec<Action>)
    path: Vec<Action>,
}

impl From<(Turn, Observation, Vec<Action>)> for Recall {
    fn from((hero, seen, path): (Turn, Observation, Vec<Action>)) -> Self {
        Self { hero, seen, path }
    }
}

impl IntoIterator for Recall {
    type Item = Game;
    type IntoIter = std::vec::IntoIter<Game>;
    fn into_iter(self) -> Self::IntoIter {
        let root = self.root();
        let genesis = std::iter::once(root);
        let history = self
            .path
            .into_iter()
            .scan(root, |g, a| Some(std::mem::replace(g, g.apply(a))));
        genesis.chain(history).collect::<Vec<_>>().into_iter();
        todo!("fencepost error?")
    }
}

impl Recall {
    pub fn new(seen: Observation, hero: Turn) -> Self {
        Self {
            seen,
            hero,
            path: Game::blinds(),
        }
    }

    pub fn root(&self) -> Game {
        Game::root().wipe(Hole::from(self.seen))
    }

    pub fn head(&self) -> Game {
        self.path
            .iter()
            .cloned()
            .fold(self.root(), |game, action| game.apply(action))
    }

    pub fn path(&self) -> Path {
        self.clone()
            .into_iter()
            .zip(self.path.iter().copied())
            .map(|(g, a)| g.edgify(a))
            .collect::<Path>()
    }

    pub fn isomorphism(&self) -> Isomorphism {
        Isomorphism::from(self.seen)
    }

    pub fn undo(&mut self) {
        if self.can_rewind() {
            self.path.pop();
        }
        while self.can_revoke() {
            self.path.pop();
        }
    }
    pub fn push(&mut self, action: Action) {
        if self.can_extend(&action) {
            self.path.push(action);
        }
        while self.can_reveal() {
            let street = self.head().street();
            let reveal = self
                .seen
                .public()
                .clone()
                .skip(street.n_observed())
                .take(street.n_revealed())
                .collect::<Vec<Card>>()
                .into();
            self.path.push(Action::Draw(reveal));
        }
    }

    pub fn can_extend(&self, action: &Action) -> bool {
        self.head().is_allowed(action)
    }
    pub fn can_rewind(&self) -> bool {
        self.path.iter().any(|a| !a.is_blind())
    }
    pub fn can_lookup(&self) -> bool {
        true
            && self.head().turn() == self.hero //               is it our turn right now?
            && self.head().street() == self.seen.street() //    have we exhausted info from Obs?
    }
    pub fn can_reveal(&self) -> bool {
        true
            && self.head().turn() == Turn::Chance //            is it time to reveal the next card?
            && self.head().street() < self.seen.street() //     would revealing double-deal?
    }
    pub fn can_revoke(&self) -> bool {
        matches!(self.path.last().expect("empty path"), Action::Draw(_))
    }
}
