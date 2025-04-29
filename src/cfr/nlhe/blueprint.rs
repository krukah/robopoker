use crate::cards::street::Street;
use crate::cfr::nlhe::encoder::Encoder;
use crate::cfr::nlhe::profile::Profile;
use crate::cfr::traits::trainer::Trainer;
use crate::save::disk::Disk;
use crate::Arbitrary;

pub struct Blueprint {
    pub(super) sampler: Encoder,
    pub(super) profile: Profile,
}

impl Blueprint {
    pub fn train() {
        if Self::done(Street::random()) {
            log::info!("resuming regret minimization from checkpoint");
            Self::load(Street::random()).solve().save();
        } else {
            log::info!("starting regret minimization from scratch");
            Self::grow(Street::random()).solve().save();
        }
    }
    pub fn discount(&self, regret: Option<crate::Utility>) -> f32 {
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

    /// Discount parameters for the training process.
    /// These values control how quickly the algorithm converges
    /// and how much weight is given to recent updates versus historical data.
    ///
    /// - `alpha`: Controls the rate at which recent updates are given more weight.
    /// - `omega`: Controls the rate at which historical updates are given more weight.
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

#[cfg(feature = "native")]
impl Disk for Blueprint {
    fn name() -> String {
        unimplemented!()
    }
    fn done(street: Street) -> bool {
        Profile::done(street) && Encoder::done(street)
    }
    fn save(&self) {
        self.profile.save();
    }
    fn grow(_: Street) -> Self {
        Self {
            profile: Profile::default(),
            sampler: Encoder::load(Street::random()),
        }
    }
    fn load(_: Street) -> Self {
        Self {
            profile: Profile::load(Street::random()),
            sampler: Encoder::load(Street::random()),
        }
    }
}
