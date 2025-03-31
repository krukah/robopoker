// todo
// belongs in ::gameplay crate ?

use crate::cards::card::Card;
use crate::cards::hole::Hole;
use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::gameplay::action::Action;
use crate::gameplay::game::Game;
use crate::gameplay::ply::Turn;
use crate::mccfr::bucket::Bucket;
use crate::mccfr::path::Path;

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

    /// lossy conversion from granular Action to coarse Edge.
    /// we depend on the pot size as of the Game state where
    /// the Action is applied, and always compare the size of the
    /// Action::Raise(_) to the pot to yield an [Odds] value.
    fn pseudoharmonics(&self) -> Path {
        todo!(
            "use pseudo-harmonic
mapping to
                    convert history:
        Recall -> Vec<(Game, Action)> -> Vec<Edge> -> Path"
        )
    }

    fn choices(&self) -> Path {
        Path::from(
            self.head()
                .choices(self.path.iter().filter(|a| a.is_aggro()).count()),
        )
    }
    pub fn bucket(&self, abstraction: Abstraction) -> Bucket {
        let present = abstraction;
        let history = Path::from(self.pseudoharmonics());
        let choices = Path::from(self.choices());
        Bucket::from((history, present, choices))
    }
}
