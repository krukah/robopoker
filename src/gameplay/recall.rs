use super::action::Action;
use super::game::Game;
use super::path::Path;
use super::turn::Turn;
use crate::cards::card::Card;
use crate::cards::hand::Hand;
use crate::cards::hole::Hole;
use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;

/// a complete representation of perfect recall [Game] history
/// from the perspective of the hero [Turn].
///
/// note that this struct implicitly assumes:
/// - default stacks
/// - default dealer position
/// - blinds, draws, and player actions are all included in action path
#[derive(Debug, Clone)]
pub struct Recall {
    turn: Turn,
    seen: Observation, // could be replaced by Hole + Board + BetHistory(Vec<Action>)
    past: Vec<Action>,
}

impl From<(Turn, Observation, Vec<Action>)> for Recall {
    fn from((turn, seen, past): (Turn, Observation, Vec<Action>)) -> Self {
        Self { turn, seen, past }
    }
}

impl Recall {
    pub fn root(&self) -> Game {
        Game::root().wipe(Hole::from(self.seen))
    }
    pub fn head(&self) -> Game {
        self.past.iter().fold(self.root(), |g, a| g.apply(*a))
    }
    #[rustfmt::skip]
    pub fn path(&self) -> Path {
        assert!(self.consistent());
        self.past
            .iter()
            .scan(self.root(), |g, a| Some(std::mem::replace(g, g.apply(*a)).edgify(*a)))
            .collect::<Path>()
        // .skip(2) 
        // .skip_while(|a| a.is_blind())
    }
    pub fn isomorphism(&self) -> Isomorphism {
        Isomorphism::from(self.seen)
    }

    pub fn consistent(&self) -> bool {
        self.seen.public().clone()
            == self
                .past
                .iter()
                .filter_map(|a| a.hand())
                .fold(Hand::empty(), Hand::add)
    }
}

#[allow(dead_code)]
impl Recall {
    fn undo(&mut self) {
        if self.can_rewind() {
            self.past.pop();
        }
        while self.can_revoke() {
            self.past.pop();
        }
    }
    fn push(&mut self, action: Action) {
        if self.can_extend(&action) {
            self.past.push(action);
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
            self.past.push(Action::Draw(reveal));
        }
    }
    fn can_extend(&self, action: &Action) -> bool {
        self.head().is_allowed(action)
    }
    fn can_rewind(&self) -> bool {
        self.past.iter().any(|a| !a.is_blind())
    }
    fn can_revoke(&self) -> bool {
        matches!(self.past.last().expect("empty path"), Action::Draw(_))
    }
    fn can_lookup(&self) -> bool {
        true
            && self.head().turn() == self.turn //               is it our turn right now?
            && self.head().street() == self.seen.street() //    have we exhausted info from Obs?
    }
    fn can_reveal(&self) -> bool {
        true
            && self.head().turn() == Turn::Chance //            is it time to reveal the next card?
            && self.head().street() < self.seen.street() //     would revealing double-deal?
    }
}
