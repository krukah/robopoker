use super::support::Support;
use fulcrum::Probability;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::Hash;

/// A discrete probability distribution over a support set.
///
/// Provides access to probability mass at each point and iteration over
/// the support. This abstraction enables optimal transport algorithms to
/// work with any collection type that maps elements to probabilities.
///
/// # Required Methods
///
/// - [`density`](Density::density) — Query probability at a point
/// - [`support`](Density::support) — Iterate over points with positive mass
///
/// # Implementations
///
/// Provided for common collection types:
/// - `BTreeMap<T, Probability>` — Ordered map with O(log n) lookup
/// - `HashMap<T, Probability>` — Hash map with O(1) expected lookup
/// - `Vec<(T, Probability)>` — Association list with O(n) lookup
pub trait Density {
    /// The type of elements in the distribution's support.
    type Support: Support;
    /// Returns the probability mass at point `x`, or 0 if not in support.
    fn density(&self, x: &Self::Support) -> Probability;
    /// Iterates over all points with positive probability mass.
    fn support(&self) -> impl Iterator<Item = Self::Support>;
}

impl<T> Density for BTreeMap<T, Probability>
where
    T: Eq + Ord + Support,
{
    type Support = T;

    fn density(&self, x: &Self::Support) -> Probability {
        self.get(x).copied().unwrap_or(0.)
    }

    fn support(&self) -> impl Iterator<Item = Self::Support> {
        self.keys().cloned()
    }
}

// Density impl is consumed via concrete HashMap construction; generic hasher param would propagate noise.
#[allow(clippy::implicit_hasher)]
impl<T> Density for HashMap<T, Probability>
where
    T: Eq + Hash + Support,
{
    type Support = T;

    fn density(&self, x: &Self::Support) -> Probability {
        self.get(x).copied().unwrap_or(0.)
    }

    fn support(&self) -> impl Iterator<Item = Self::Support> {
        self.keys().cloned()
    }
}

impl<T> Density for Vec<(T, Probability)>
where
    T: Eq + Support,
{
    type Support = T;

    fn density(&self, x: &Self::Support) -> Probability {
        self.iter().find(|(a, _)| a == x).map(|(_, p)| p).copied().unwrap_or(0.)
    }

    fn support(&self) -> impl Iterator<Item = Self::Support> {
        self.iter().map(|(a, _)| a).cloned()
    }
}
