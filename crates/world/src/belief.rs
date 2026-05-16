//! Secret-to-world classification with per-world probability weights.
//!
//! Combines the secret → world mapping (for rejection sampling) with the
//! K-world weight distribution (for the subgame gadget). Produced by
//! [`Partition::partition`] and consumed by [`SubGameEncoder`] and [`SubProfile`].
use crate::World;
use rbp_core::Probability;
use rbp_mccfr::CfrSecret;
use rbp_transport::Density;
use std::collections::BTreeMap;

/// Maps each opponent secret to its assigned world and tracks per-world weights.
///
/// This is a discretization of the posterior: each secret belongs to exactly
/// one quantile bucket (world), and each world carries its share of total
/// probability mass. Contrast with [`rbp_mccfr::Posterior<Y>`] which maps secrets to
/// continuous reach probabilities.
#[derive(Debug, Clone)]
pub struct Belief<Y, const W: usize>
where
    Y: CfrSecret,
{
    members: BTreeMap<Y, World>,
    weights: [Probability; W],
}

impl<Y, const W: usize> Belief<Y, W>
where
    Y: CfrSecret,
{
    /// Constructs a belief from a mapping and weight array.
    pub fn new(members: BTreeMap<Y, World>, weights: [Probability; W]) -> Self {
        Self { members, weights }
    }
    /// Which world does this secret belong to?
    pub fn world(&self, secret: &Y) -> Option<World> {
        self.members.get(secret).copied()
    }
    /// Does this secret belong to the target world?
    ///
    /// Returns false if the secret is unclassified...but...
    /// Returns true if the belief has no classifications.
    pub fn remember(&self, secret: &Y, world: World) -> bool {
        self.members.is_empty() || self.world(secret).is_some_and(|w| w == world)
    }
    /// Per-world weight array for SubProfile construction.
    pub fn weights(&self) -> [Probability; W] {
        self.weights
    }
}

impl<Y, const W: usize> Density for Belief<Y, W>
where
    Y: CfrSecret,
{
    type Support = World;

    fn density(&self, world: &World) -> Probability {
        self.weights.get(world.index()).copied().unwrap_or(0.0)
    }

    fn support(&self) -> impl Iterator<Item = World> {
        (0..W).map(World::from)
    }
}

// impl<Y, const W: usize> FromIterator<(Y, World)> for Belief<Y, W>
// where
//     Y: CfrSecret,
// {
//     fn from_iter<I>(iter: I) -> Self
//     where
//         I: IntoIterator<Item = (Y, World)>,
//     {
//         Self {
//             members: iter.into_iter().collect(),
//             weights: [0.0; W],
//         }
//     }
// }
