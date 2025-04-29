use crate::cfr::traits::profile::Profile;
use crate::cfr::traits::trainer::Trainer;

impl Trainer for super::blueprint::Blueprint {
    type T = super::turn::Turn;
    type E = super::edge::Edge;
    type G = super::game::Game;
    type I = super::info::Info;
    type P = super::profile::Profile;
    type S = super::encoder::Encoder;

    fn advance(&mut self) {
        self.profile.increment();
    }
    fn encoder(&self) -> &Self::S {
        &self.sampler
    }
    fn profile(&self) -> &Self::P {
        &self.profile
    }
    fn policy(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.profile.at(info.clone(), edge.clone()).0
    }
    fn regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.profile.at(info.clone(), edge.clone()).1
    }
    fn discount(&self, regret: Option<crate::Utility>) -> f32 {
        self.discount(regret)
    }
}
