use std::hash::Hash;

/// An element of the finite set of possible actions.
pub(crate) trait Action: Eq + Hash + Copy {}
