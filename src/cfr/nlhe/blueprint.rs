use crate::cfr::traits::blueprint::Blueprint;
use crate::cfr::traits::profile::Profile;
use std::collections::BTreeMap;

impl Blueprint for super::solver::NLHE {
    type T = super::turn::Turn;
    type E = super::edge::Edge;
    type G = super::game::Game;
    type I = super::info::Info;
    type P = super::profile::Profile;
    type S = super::encoder::Encoder;

    fn train() {
        use crate::cards::street::Street;
        use crate::save::disk::Disk;
        use crate::Arbitrary;
        if Self::done(Street::random()) {
            log::info!("resuming regret minimization from checkpoint");
            Self::load(Street::random()).solve().save();
        } else {
            log::info!("starting regret minimization from scratch");
            Self::grow(Street::random()).solve().save();
        }
    }

    fn tree_count() -> usize {
        crate::CFR_TREE_COUNT_NLHE
    }
    fn batch_size() -> usize {
        crate::CFR_BATCH_SIZE_NLHE
    }

    fn advance(&mut self) {
        self.profile.increment();
    }
    fn encoder(&self) -> &Self::S {
        &self.sampler
    }
    fn profile(&self) -> &Self::P {
        &self.profile
    }
    fn mut_policy(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .profile
            .encounters
            .entry(*info)
            .or_insert_with(BTreeMap::default)
            .entry(*edge)
            .or_insert_with(|| (0., 0.))
            .0
    }
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .profile
            .encounters
            .entry(*info)
            .or_insert_with(BTreeMap::default)
            .entry(*edge)
            .or_insert_with(|| (0., 0.))
            .1
    }
}
