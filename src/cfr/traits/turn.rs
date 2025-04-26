/// Represents a player turn is fully abstracted. it is almost a marker trait,
/// but requires two fundamnetal properties:
/// 1. it has an element of chance
/// 2. its elements can be mapped to by unsigned integers
pub trait Turn: Clone + Copy + PartialEq + Eq + std::fmt::Debug + std::hash::Hash {}
