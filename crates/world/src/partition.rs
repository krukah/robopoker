//! Extension trait for discretizing posteriors into belief partitions.
//!
//! Sorts secrets by reach probability, partitions into K segments of equal
//! probability mass, and returns each segment's weight alongside the
//! secret-to-world classification. World 0 contains the highest-reach
//! secrets, world K-1 the lowest.
use crate::Belief;
use crate::World;
use rbp_core::Probability;
use rbp_mccfr::CfrSecret;
use rbp_mccfr::Posterior;
use std::collections::BTreeMap;

/// Discretizes a posterior distribution into K quantile-weighted worlds.
pub trait Partition<Y>
where
    Y: CfrSecret,
{
    fn partition<const W: usize>(self) -> Belief<Y, W>;
}

impl<Y> Partition<Y> for Posterior<Y>
where
    Y: CfrSecret,
{
    fn partition<const W: usize>(self) -> Belief<Y, W> {
        let total = self.total();
        if total <= 0.0 {
            let mapping = self.into_iter().map(|(s, _)| (s, World::from(0))).collect();
            return Belief::new(mapping, [1.0 / W as Probability; W]);
        }
        let mut sorted = self.into_iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let segment = total / W as Probability;
        let mut weights = [0.0; W];
        let mut mapping = BTreeMap::new();
        let mut index = 0usize;
        let mut bucket = 0.0;
        let mut accumulated = 0.0;
        for (secret, reach) in sorted {
            bucket += reach;
            accumulated += reach;
            mapping.insert(secret, World::from(index));
            if accumulated >= segment * (index + 1) as Probability && index < W - 1 {
                weights[index] = bucket / total;
                index += 1;
                bucket = 0.0;
            }
        }
        weights[index] = bucket / total;
        Belief::new(mapping, weights)
    }
}
