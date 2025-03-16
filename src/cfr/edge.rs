/// Represents an edge in the game tree
pub trait Edge: Clone + Copy + PartialEq + Eq + std::fmt::Debug {}

impl Edge for petgraph::graph::EdgeIndex {}
impl Edge for crate::mccfr::edge::Edge {}
