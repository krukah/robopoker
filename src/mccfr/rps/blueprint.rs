use super::edge::Edge;
use super::game::Game;
use super::solver::RPS;
use super::turn::Turn;
use std::collections::BTreeMap;

/// For the Rock Paper Scissors game, Blueprint implements both Trainer and Profile traits.
/// As a Profile, it tracks regrets and policies over time. As a Trainer, it uses those
/// values to train an optimal strategy through counterfactual regret minimization.
impl crate::mccfr::traits::blueprint::Blueprint for RPS {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;
    type P = Self;
    type S = Self;

    fn train() {
        log::info!("{}", Self::default().solve());
    }

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

    fn mut_policy(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(info.clone())
            .or_insert_with(BTreeMap::default)
            .entry(edge.clone())
            .or_insert((0., 0.))
            .0
    }

    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(info.clone())
            .or_insert_with(BTreeMap::default)
            .entry(edge.clone())
            .or_insert((0., 0.))
            .1
    }

    fn advance(&mut self) {
        crate::mccfr::traits::profile::Profile::increment(self)
    }
}
