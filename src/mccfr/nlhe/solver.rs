use super::*;
use crate::gameplay::*;
use crate::mccfr::*;
use crate::*;
use std::collections::BTreeMap;

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
pub struct NlheSolver {
    pub sampler: NlheEncoder,
    pub profile: NlheProfile,
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl crate::save::Hydrate for NlheSolver {
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        Self {
            sampler: NlheEncoder::hydrate(client.clone()).await,
            profile: NlheProfile::hydrate(client.clone()).await,
        }
    }
}

impl Blueprint for NlheSolver {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Info;
    type P = NlheProfile;
    type S = NlheEncoder;

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
    fn mut_policy(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        &mut self
            .profile
            .encounters
            .entry(*info)
            .or_insert_with(BTreeMap::default)
            .entry(*edge)
            .or_insert_with(|| edge.policy())
            .0
    }
    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        &mut self
            .profile
            .encounters
            .entry(*info)
            .or_insert_with(BTreeMap::default)
            .entry(*edge)
            .or_insert_with(|| edge.regret())
            .1
    }
}

#[cfg(feature = "disk")]
use crate::cards::*;

#[cfg(feature = "disk")]
#[allow(deprecated)]
impl NlheSolver {
    #[allow(unused)]
    fn write() {
        use crate::save::Disk;
        if Self::done(Street::random()) {
            log::info!("resuming regret minimization from checkpoint");
            Self::load(Street::random()).solve().save();
        } else {
            log::info!("starting regret minimization from scratch");
            Self::grow(Street::random()).solve().save();
        }
    }
}

#[cfg(feature = "disk")]
#[allow(deprecated)]
impl crate::save::Disk for NlheSolver {
    fn name() -> &'static str {
        "solver"
    }
    fn sources() -> Vec<std::path::PathBuf> {
        Vec::new()
    }
    fn done(street: Street) -> bool {
        NlheProfile::done(street) && NlheEncoder::done(street)
    }
    fn save(&self) {
        self.profile.save();
    }
    fn grow(_: Street) -> Self {
        Self {
            profile: NlheProfile::default(),
            sampler: NlheEncoder::load(Street::random()),
        }
    }
    fn load(_: Street) -> Self {
        Self {
            profile: NlheProfile::load(Street::random()),
            sampler: NlheEncoder::load(Street::random()),
        }
    }
}
