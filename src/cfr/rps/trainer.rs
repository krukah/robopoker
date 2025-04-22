use super::blueprint::Blueprint;
use super::edge::Edge;
use super::game::Game;
use super::sampler::Sampler;
use super::turn::Turn;

impl crate::cfr::traits::trainer::Trainer for Blueprint {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;
    type P = Blueprint;
    type S = Sampler;

    fn encoder(&self) -> &Self::S {
        &Sampler
    }

    fn profile(&self) -> &Self::P {
        self
    }

    fn discount(&self, _: Option<crate::Utility>) -> f32 {
        1.
    }

    fn regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.at(info, edge).1
    }

    fn weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.at(info, edge).0
    }
}
