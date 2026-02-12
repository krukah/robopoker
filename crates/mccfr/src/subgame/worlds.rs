//! K-world distribution for safe subgame solving.
//!
//! The "worlds" represent clustered opponent range buckets used in the
//! subgame gadget construction. At the subgame root, the opponent chooses
//! which world to enter, weighted by how likely they are to hold hands
//! in that bucket.
//!
//! # From Pluribus Paper
//!
//! Safe subgame solving requires the opponent to choose among K "alternative
//! worlds" at the subgame root. Each world represents a different slice of
//! the opponent's possible hand range, weighted by blueprint reach.
//!
//! This construction prevents exploitation: the solver must handle all
//! worlds, so it's robust even if the opponent deviated from blueprint
//! before entering the subgame.
//!
//! # Statistical Interpretation
//!
//! Worlds implement **quantile-based discretization** of the posterior:
//!
//! ```text
//! Reaches: [hand₁: 0.15, hand₂: 0.12, ...]  (continuous posterior)
//!     ↓ sort by probability, partition into K equal-mass segments
//! Worlds:  [0.25, 0.25, 0.25, 0.25]         (K discrete buckets)
//! ```
//!
//! This is analogous to **distributionally robust optimization**: instead
//! of committing to one belief about opponent's range, we hedge against
//! K discretized possibilities. The resulting strategy is minimax-optimal
//! over the set of possible opponent priors.
//!
//! # Clustering
//!
//! The K worlds are created by:
//! 1. Sorting opponent hands/abstractions by reach probability
//! 2. Partitioning into K segments of equal probability mass
//! 3. Each world's weight is its segment's total mass
use crate::Posterior;
use rbp_core::Probability;
use rbp_transport::*;

/// Quantile-discretized distribution over K alternative worlds.
///
/// Each world represents a reach-weighted bucket of opponent private info,
/// partitioned by equal probability mass. World 0 contains highest-reach
/// values, world K-1 contains lowest. Used in the subgame gadget to
/// achieve distributionally robust strategies.
#[derive(Debug, Clone, Copy)]
pub struct ManyWorlds<const K: usize>([Probability; K]);

impl<const K: usize> ManyWorlds<K> {
    /// Discretizes a belief distribution over opponent secrets into K worlds.
    ///
    /// # Prior Construction (Caller's Responsibility)
    ///
    /// The `Posterior<P>` input represents our belief about the opponent's
    /// private information given observed actions. Construction is **game-specific**
    /// and lives outside the generic CFR abstractions. Typical steps:
    ///
    /// 1. **Enumerate** all possible opponent secrets (e.g., hole card combinations)
    /// 2. **Bind** each secret to public info to make a complete-information history
    /// 3. **Compute reach** via `Solver::external_reach()` (product of blueprint
    ///    action probabilities along the observed path)
    /// 4. **Collect** into `Posterior<P>` via `FromIterator<(P, Probability)>`
    /// 5. **Project** to coarser granularity if needed (e.g., observations → buckets)
    ///
    /// The generic pipeline imposes no contract on *how* this distribution is
    /// obtained — only that it exists and is keyed by some `CfrSecret` type.
    ///
    /// # Quantile Bucketing
    ///
    /// Sorts secrets by reach probability, partitions into K segments of equal
    /// probability mass, and returns each segment's weight. World 0 contains
    /// the highest-reach secrets, world K-1 the lowest.
    pub fn cluster<P>(reaches: Posterior<P>) -> Self
    where
        P: crate::CfrSecret,
    {
        let total = reaches.total();
        if total <= 0.0 {
            return Self::uniform();
        }
        let mut sorted = reaches.into_iter().map(|(_, p)| p).collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let segment = total / K as Probability;
        let mut worlds = [0.0; K];
        let mut index = 0;
        let mut bucket = 0.0;
        let mut accumulated = 0.0;
        for reach in sorted {
            bucket += reach;
            accumulated += reach;
            if accumulated >= segment * (index + 1) as Probability && index < K - 1 {
                worlds[index] = bucket / total;
                index += 1;
                bucket = 0.0;
            }
        }
        worlds[index] = bucket / total;
        Self(worlds)
    }
    /// Creates a uniform distribution over K worlds.
    pub fn uniform() -> Self {
        Self([1.0 / K as Probability; K])
    }
    /// Weight of a specific world.
    pub fn weight(&self, world: usize) -> Probability {
        self.0.get(world).copied().unwrap_or(0.0)
    }
}

impl<const K: usize> Density for ManyWorlds<K> {
    type Support = usize;
    fn density(&self, world: &usize) -> Probability {
        self.weight(*world)
    }
    fn support(&self) -> impl Iterator<Item = Self::Support> {
        0..K
    }
}

impl<const K: usize> From<ManyWorlds<K>> for [Probability; K] {
    fn from(worlds: ManyWorlds<K>) -> Self {
        worlds.0
    }
}

impl<const K: usize> std::fmt::Display for ManyWorlds<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, w) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:.3}", w)?;
        }
        write!(f, "]")
    }
}
