pub struct Trainer {
    sampler: crate::cfr::nlhe::sampler::Sampler,
    profile: crate::cfr::nlhe::profile::Profile,
}

impl Trainer {
    pub fn train() {
        use crate::cards::street::Street;
        use crate::cfr::traits::trainer::Trainer;
        use crate::save::disk::Disk;
        use crate::Arbitrary;
        let mut solution = Self::load(Street::random());
        solution.solve();
        solution.save();
    }
    const fn alpha(&self) -> f32 {
        1.5
    }
    const fn omega(&self) -> f32 {
        0.5
    }
    const fn gamma(&self) -> f32 {
        1.5
    }
    const fn period(&self) -> usize {
        1
    }
}

impl crate::cfr::traits::trainer::Trainer for Trainer {
    type T = crate::cfr::nlhe::turn::Turn;
    type E = crate::cfr::nlhe::edge::Edge;
    type G = crate::cfr::nlhe::game::Game;
    type I = crate::cfr::nlhe::info::Info;
    type P = crate::cfr::nlhe::profile::Profile;
    type S = crate::cfr::nlhe::sampler::Sampler;

    fn advance(&mut self) {
        use crate::cfr::traits::profile::Profile;
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

impl std::fmt::Display for Trainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.profile)
    }
}

#[cfg(feature = "native")]
impl crate::save::disk::Disk for Trainer {
    fn name() -> String {
        unimplemented!()
    }
    fn done(street: crate::cards::street::Street) -> bool {
        crate::cfr::nlhe::profile::Profile::done(street)
            && crate::cfr::nlhe::sampler::Sampler::done(street)
    }

    fn save(&self) {
        self.profile.save();
        self.sampler.save();
    }

    fn grow(_: crate::cards::street::Street) -> Self {
        use crate::cards::street::Street;
        use crate::Arbitrary;
        Self {
            profile: crate::cfr::nlhe::profile::Profile::default(),
            sampler: crate::cfr::nlhe::sampler::Sampler::load(Street::random()),
        }
    }

    fn load(_: crate::cards::street::Street) -> Self {
        use crate::cards::street::Street;
        use crate::Arbitrary;
        Self {
            profile: crate::cfr::nlhe::profile::Profile::load(Street::random()),
            sampler: crate::cfr::nlhe::sampler::Sampler::load(Street::random()),
        }
    }
}
