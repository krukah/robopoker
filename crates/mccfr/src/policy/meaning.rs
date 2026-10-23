//! What the numbers in a [`Over`](crate::Over) mean.

/// The semantic tag on a per-edge scalar vector. A closed vocabulary of
/// two markers, carrying no data — they exist so that a regret vector and
/// a strategy vector, which share a representation, stop sharing a type.
pub trait Meaning {}

/// Unnormalized counterfactual regret increments. May be negative; does
/// not sum to anything in particular.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Regret;

/// A distribution over an infoset's edges. Non-negative, sums to one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Policy;

impl Meaning for Regret {}

impl Meaning for Policy {}
