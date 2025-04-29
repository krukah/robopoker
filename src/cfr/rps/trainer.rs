use super::edge::Edge;
use super::encoder::Encoder;
use super::game::Game;
use super::turn::Turn;
use crate::cfr::rps::blueprint::Blueprint;
use crate::cfr::traits::profile::Profile;
use crate::cfr::traits::trainer::Trainer;

/// For the Rock Paper Scissors game, Blueprint implements both Trainer and Profile traits.
/// As a Profile, it tracks regrets and policies over time. As a Trainer, it uses those
/// values to train an optimal strategy through counterfactual regret minimization.
impl Trainer for Blueprint {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;
    type P = Blueprint;
    type S = Encoder;

    fn encoder(&self) -> &Self::S {
        &Encoder
    }

    fn profile(&self) -> &Self::P {
        &self
    }

    fn discount(&self, regret: Option<crate::Utility>) -> f32 {
        self.discount(regret)
    }

    fn policy(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.at(info, edge).0
    }

    fn regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.at(info, edge).1
    }

    fn advance(&mut self) {
        Profile::increment(self);
    }
}
