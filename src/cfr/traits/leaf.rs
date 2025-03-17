/// Represents a leaf node in the game tree
pub trait Leaf: Clone + Copy {}

impl Leaf for crate::mccfr::leaf::Leaf {}
