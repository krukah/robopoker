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
        use crate::cfr::traits::profile::Profile;
        match regret {
            None => {
                let g = self.gamma();
                let t = self.profile().epochs() as f32;
                (t / (t + 1.)).powf(g)
            }
            Some(r) => {
                let a = self.alpha();
                let o = self.omega();
                let p = self.period() as f32;
                let t = self.profile().epochs() as f32;
                if t % p != 0. {
                    1.
                } else if r > 0. {
                    let x = (t / p).powf(a);
                    x / (x + 1.)
                } else if r < 0. {
                    let x = (t / p).powf(o);
                    x / (x + 1.)
                } else {
                    let x = t / p;
                    x / (x + 1.)
                }
            }
        }
    }
}
