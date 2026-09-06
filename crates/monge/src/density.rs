use super::support::Support;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::Hash;

/// A discrete probability distribution over a support set, so the transport
/// algorithms can run against any collection mapping elements to
/// probabilities. Implemented below for `BTreeMap`, `HashMap`, and
/// `Vec<(T, f32)>`.
pub trait Density {
    /// The type of elements in the distribution's support.
    type Support: Support;
    /// Returns the probability mass at point `x`, or 0 if not in support.
    fn density(&self, x: &Self::Support) -> f32;
    /// Iterates over all points with positive probability mass.
    fn support(&self) -> impl Iterator<Item = Self::Support>;
}

impl<T> Density for BTreeMap<T, f32>
where
    T: Eq + Ord + Support,
{
    type Support = T;

    fn density(&self, x: &Self::Support) -> f32 {
        self.get(x).copied().unwrap_or(0.)
    }

    fn support(&self) -> impl Iterator<Item = Self::Support> {
        self.keys().cloned()
    }
}

// Density impl is consumed via concrete HashMap construction; generic hasher param would propagate noise.
#[allow(clippy::implicit_hasher)]
impl<T> Density for HashMap<T, f32>
where
    T: Eq + Hash + Support,
{
    type Support = T;

    fn density(&self, x: &Self::Support) -> f32 {
        self.get(x).copied().unwrap_or(0.)
    }

    fn support(&self) -> impl Iterator<Item = Self::Support> {
        self.keys().cloned()
    }
}

impl<T> Density for Vec<(T, f32)>
where
    T: Eq + Support,
{
    type Support = T;

    fn density(&self, x: &Self::Support) -> f32 {
        self.iter().find(|(a, _)| a == x).map(|(_, p)| p).copied().unwrap_or(0.)
    }

    fn support(&self) -> impl Iterator<Item = Self::Support> {
        self.iter().map(|(a, _)| a).cloned()
    }
}
