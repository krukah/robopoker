//! Aggregated feature frequencies for one bucket.
use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;
use std::collections::BTreeMap;

/// A bucket seen through the [`Feature`] vocabulary: how often each predicate
/// held across a uniform sample of its members, alongside the scalar
/// [`Census`] facts.
///
/// The sample is what keeps this cheap. A turn bucket can hold a hundred
/// thousand isomorphisms; a hundred of them pin every frequency to within a
/// few points, which is finer than any label needs.
pub struct Profile {
    census: Census,
    tally: BTreeMap<Feature, usize>,
    sampled: usize,
    exemplars: Vec<Observation>,
}

impl Profile {
    /// How many exemplars a rendered description carries.
    pub const EXEMPLARS: usize = 3;

    pub fn census(&self) -> &Census {
        &self.census
    }

    pub fn abs(&self) -> Abstraction {
        self.census.abs()
    }

    pub fn sampled(&self) -> usize {
        self.sampled
    }

    pub fn exemplars(&self) -> &[Observation] {
        &self.exemplars
    }

    /// Fraction of sampled members that carry a feature.
    pub fn frequency(&self, feature: Feature) -> Probability {
        match self.sampled {
            0 => 0.,
            n => self.tally.get(&feature).copied().unwrap_or_default() as Probability / n as Probability,
        }
    }

    /// The most common feature in a group, with its share of the sample.
    /// Ties go to the earlier feature: declaration order is salience order,
    /// so a board that is equally paired and rainbow reads as paired.
    pub fn dominant(&self, group: &[Feature]) -> Option<(Feature, Probability)> {
        group
            .iter()
            .copied()
            .map(|feature| (feature, self.frequency(feature)))
            .filter(|(_, p)| *p > 0.)
            .fold(None, |best, (feature, p)| match best {
                Some((_, held)) if held >= p => best,
                _ => Some((feature, p)),
            })
    }

    /// The most common feature in a group, with its share *of that group* —
    /// the right denominator when a group only applies to some members, as
    /// kicker quality only applies to hands that made a pair.
    pub fn within(&self, group: &[Feature]) -> Option<(Feature, Probability)> {
        let total = group.iter().map(|f| self.frequency(*f)).sum::<Probability>();
        self.dominant(group)
            .filter(|_| total > 0.)
            .map(|(feature, p)| (feature, p / total))
    }

    /// Total frequency of a group. Above one for groups whose features can
    /// hold at once, exactly one for the exhaustive ones.
    pub fn total(&self, group: &[Feature]) -> Probability {
        group.iter().map(|feature| self.frequency(*feature)).sum()
    }

    /// Every feature in a group above a frequency floor, most common first.
    pub fn ranked(&self, group: &[Feature], floor: Probability) -> Vec<(Feature, Probability)> {
        let mut ranked = group
            .iter()
            .copied()
            .map(|feature| (feature, self.frequency(feature)))
            .filter(|(_, p)| *p >= floor)
            .collect::<Vec<_>>();
        ranked.sort_by(|(_, a), (_, b)| b.total_cmp(a));
        ranked
    }

    /// The same ranking, renormalized so the shares are of the group rather
    /// than of the sample — how kicker quality wants to be read.
    pub fn conditioned(&self, group: &[Feature], floor: Probability) -> Vec<(Feature, Probability)> {
        match self.total(group) {
            0. => Vec::new(),
            total => self
                .ranked(group, floor * total)
                .iter()
                .map(|(feature, p)| (*feature, p / total))
                .collect(),
        }
    }
}

impl From<(Census, Vec<Observation>)> for Profile {
    fn from((census, sample): (Census, Vec<Observation>)) -> Self {
        Self {
            census,
            sampled: sample.len(),
            exemplars: sample.iter().take(Self::EXEMPLARS).copied().collect(),
            tally: sample
                .iter()
                .flat_map(Feature::of)
                .fold(BTreeMap::<Feature, usize>::new(), |mut tally, feature| {
                    *tally.entry(feature).or_default() += 1;
                    tally
                }),
        }
    }
}
