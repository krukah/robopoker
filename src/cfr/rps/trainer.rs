use super::blueprint::Blueprint;
use super::edge::Edge;
use super::game::Game;
use super::turn::Turn;

/// For the Rock Paper Scissors game, Blueprint implements both Trainer and Profile traits.
/// As a Profile, it tracks regrets and policies over time. As a Trainer, it uses those
/// values to train an optimal strategy through counterfactual regret minimization.
impl crate::cfr::traits::trainer::Trainer for Blueprint {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;
    type P = Self;
    type S = Self;

    fn tree_count() -> usize {
        crate::CFR_TREE_COUNT_RPS
    }
    fn batch_size() -> usize {
        crate::CFR_BATCH_SIZE_RPS
    }

    fn encoder(&self) -> &Self::S {
        &self
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
        use crate::cfr::traits::profile::Profile;
        self.increment()
    }
}
