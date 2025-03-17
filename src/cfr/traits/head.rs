/// Represents a head node in the game tree
pub trait Head: Clone + Copy + PartialEq + Eq {}

impl Head for petgraph::graph::NodeIndex {}
