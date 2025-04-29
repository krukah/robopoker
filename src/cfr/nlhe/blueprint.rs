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
        let mut solution = Self::load(Street::random());
        solution.solve();
        solution.save();
    }
    pub const fn alpha(&self) -> f32 {
        1.5
    }
    pub const fn omega(&self) -> f32 {
        0.5
    }
    pub const fn gamma(&self) -> f32 {
        1.5
    }
    pub const fn period(&self) -> usize {
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
        self.sampler.save();
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

impl std::fmt::Display for Blueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.profile)
    }
}
