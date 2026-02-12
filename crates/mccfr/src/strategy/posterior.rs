//! Generic posterior distribution over private information.
//!
//! A posterior represents a Bayesian belief over opponent's private information
//! given observed actions. The `Posterior<P>` type unifies:
//!
//! - `Range = Posterior<Observation>` — hole-level distribution
//! - `Reach = Posterior<Abstraction>` — abstraction-level distribution
//!
//! # Bayesian Interpretation
//!
//! ```text
//! P(private | actions) ∝ P(actions | private) × P(private)
//!                        ↑                      ↑
//!                        likelihood             prior (uniform)
//! ```
//!
//! The likelihood P(actions | private) is the product of blueprint action
//! probabilities along the observed history.
use crate::CfrSecret;
use rbp_core::Probability;
use rbp_transport::Density;
use std::collections::BTreeMap;

/// Unnormalized probability distribution over private information.
///
/// Maps each possible private value to its reach probability, computed
/// by multiplying blueprint action probabilities along the observed history.
#[derive(Debug, Clone)]
pub struct Posterior<P>(BTreeMap<P, Probability>)
where
    P: CfrSecret;

impl<P> Default for Posterior<P>
where
    P: CfrSecret,
{
    fn default() -> Self {
        Self(BTreeMap::default())
    }
}

impl<P> Posterior<P>
where
    P: CfrSecret,
{
    /// Adds probability mass, returning self for chaining.
    pub fn add(mut self, private: P, prob: Probability) -> Self {
        *self.0.entry(private).or_insert(0.0) += prob;
        self
    }
    /// Mutable accumulate for fold patterns.
    pub fn accumulate(&mut self, private: P, prob: Probability) {
        *self.0.entry(private).or_insert(0.0) += prob;
    }
    /// Total probability mass in the distribution.
    pub fn total(&self) -> Probability {
        self.0.values().sum()
    }
    /// Normalizes to sum to 1.
    pub fn normalize(mut self) -> Self {
        match self.total() {
            0f32 => self, // unreachable! ?
            mass => {
                self.0.values_mut().for_each(|p| *p /= mass);
                self
            }
        }
    }
    /// Projects to coarser granularity via a mapping function.
    pub fn project<Q, F>(self, f: F) -> Posterior<Q>
    where
        Q: CfrSecret,
        F: Fn(P) -> Q,
    {
        self.into_iter()
            .fold(Posterior::default(), |post, (p, prob)| post.add(f(p), prob))
    }
}

impl<P> Density for Posterior<P>
where
    P: CfrSecret,
{
    type Support = P;
    fn density(&self, x: &P) -> Probability {
        self.0.get(x).copied().unwrap_or(0.0)
    }
    fn support(&self) -> impl Iterator<Item = P> {
        self.0.keys().copied()
    }
}

impl<P> IntoIterator for Posterior<P>
where
    P: CfrSecret,
{
    type Item = (P, Probability);
    type IntoIter = std::collections::btree_map::IntoIter<P, Probability>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<P> FromIterator<(P, Probability)> for Posterior<P>
where
    P: CfrSecret,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (P, Probability)>,
    {
        iter.into_iter()
            .fold(Self::default(), |post, (p, prob)| post.add(p, prob))
    }
}
