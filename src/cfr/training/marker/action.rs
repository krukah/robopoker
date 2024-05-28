use std::hash::Hash;

#[allow(dead_code)]
/// An element of the finite set of possible actions.
pub(crate) trait Action: Eq + Hash + Copy {}
