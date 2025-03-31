use super::support::Support;
use crate::Probability;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::Hash;

/// generalization of any probability distribution over
/// arbitrary Support.
pub trait Density {
    type Support: Support;

    fn density(&self, x: &Self::Support) -> Probability;
    fn support(&self) -> impl Iterator<Item = &Self::Support>;
}

impl<T> Density for BTreeMap<T, Probability>
where
    T: Eq + Ord + Support,
{
    type Support = T;

    fn density(&self, x: &Self::Support) -> Probability {
        self.get(x).cloned().unwrap_or(0.)
    }

    fn support(&self) -> impl Iterator<Item = &Self::Support> {
        self.keys()
    }
}

impl<T> Density for HashMap<T, Probability>
where
    T: Eq + Hash + Support,
{
    type Support = T;

    fn density(&self, x: &Self::Support) -> Probability {
        self.get(x).cloned().unwrap_or(0.)
    }

    fn support(&self) -> impl Iterator<Item = &Self::Support> {
        self.keys()
    }
}

impl<T> Density for Vec<(T, Probability)>
where
    T: Eq + Support,
{
    type Support = T;

    fn density(&self, x: &Self::Support) -> Probability {
        self.iter()
            .find(|(a, _)| a == x)
            .map(|(_, p)| p)
            .copied()
            .unwrap_or(0.)
    }

    fn support(&self) -> impl Iterator<Item = &Self::Support> {
        self.iter().map(|(a, _)| a)
    }
}
