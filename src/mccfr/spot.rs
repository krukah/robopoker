use super::policy::Policy;
use crate::cards::hole::Hole;
use crate::play::action::Action;
use crate::play::game::Game;

pub struct Spot {
    root: Game, // only used for starting stacks (hopefully)
    past: Vec<Action>,
    hole: Hole,
}
impl Spot {
    pub fn root(&self) -> &Game {
        &self.root
    }
    pub fn coalesce(&self, policy: Policy) -> Policy {
        todo!()
    }
}
