use super::edge::Turn;
use crate::transport::density::Density;
use crate::transport::support::Support;
use crate::Probability;
use std::collections::BTreeMap;
use std::iter::FromIterator;

impl<E> Policy<E> for Vec<(E, Probability)> where E: Turn + Support {}
impl<E> Policy<E> for BTreeMap<E, Probability> where E: Turn + Support + Ord {}

/// A Policy is a probability distribution over edges at a given decision point.
/// It encapsulates both the ability to query probabilities for edges (Density)
/// and the ability to construct a policy from a sequence of (Edge, Probability) pairs.
pub trait Policy<E>: Density<Support = E> + FromIterator<(E, Probability)>
where
    E: Turn,
{
}
