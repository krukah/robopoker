use crate::cards::card::Card;
use crate::cards::hole::Hole;
use crate::cards::observation::Observation;
use crate::gameplay::action::Action;
use crate::gameplay::game::Game;
use crate::gameplay::ply::Next;

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
#[derive(Debug, Clone)]
pub struct Recall {
    hero: Next,
    seen: Observation, // could be replaced by Hole + BetHistory(Vec<Action>)
    path: Vec<Action>,
}

impl Recall {
    pub fn from(seen: Observation, hero: Next) -> Self {
        Self {
            seen,
            hero,
            path: Game::blinds(),
        }
    }

    pub fn root(&self) -> Game {
        Game::root().wipe(Hole::from(self.seen))
    }

    pub fn game(&self) -> Game {
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
            let street = self.game().street();
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
        self.game().is_allowed(action)
    }
    pub fn can_rewind(&self) -> bool {
        self.path.iter().any(|a| !a.is_blind())
    }
    pub fn can_lookup(&self) -> bool {
        let game = self.game();
        true
            && game.player() == self.hero //             are we deciding right now?
            && game.street() == self.seen.street() //    have we exhausted info from Obs?
    }
    pub fn can_reveal(&self) -> bool {
        let game = self.game();
        true
            && game.player() == Next::Chance //          is it time to reveal?
            && game.street() < self.seen.street() //     would revealing double-deal?
    }
    pub fn can_revoke(&self) -> bool {
        matches!(self.path.last().expect("empty path"), Action::Draw(_))
    }
}
