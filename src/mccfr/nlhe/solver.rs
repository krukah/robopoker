use super::encoder::Encoder;
use super::profile::Profile;
use crate::cards::street::Street;
use crate::save::disk::Disk;
use crate::Arbitrary;

/// NLHE represents the complete Monte Carlo Counterfactual Regret Minimization (MCCFR) algorithm
/// for No-Limit Hold'em poker. It combines:
///
/// - An Encoder for sampling game trees and computing counterfactual values
/// - A Profile for tracking accumulated regrets and strategy weights over time
///
/// During training, it:
/// 1. Uses the Encoder to sample game situations and compute counterfactual values
/// 2. Updates the Profile's regrets and policies based on those values
/// 3. Gradually converges toward Nash equilibrium through repeated iterations
///
/// The training process uses external sampling MCCFR with alternating updates and
/// linear averaging of strategies over time.
pub struct NLHE {
    pub(super) sampler: Encoder,
    pub(super) profile: Profile,
}

impl NLHE {
    pub fn solve() {
        use crate::mccfr::traits::blueprint::Blueprint;
        Self::train();
    }
}

impl Disk for NLHE {
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
