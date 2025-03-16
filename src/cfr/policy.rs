use super::edge::Edge;
use crate::transport::density::Density;
use crate::Probability;
use std::iter::FromIterator;

/// A Policy is a probability distribution over edges at a given decision point.
/// It encapsulates both the ability to query probabilities for edges (Density)
/// and the ability to construct a policy from a sequence of (Edge, Probability) pairs.
pub trait Policy<E>: Density<Support = E> + FromIterator<(E, Probability)>
where
    E: Edge,
{
}
