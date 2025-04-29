/// the tree-local game state is fully abstracted. it must be implemented
/// by the consumer of this MCCFR API.
///
/// the implementation must be able to create a Game from:
///  scratch (i.e. root node without context)
///
/// the implementation must be able to determine:
///  whose turn is it (have a Player function)
///  how much payoff for each player (only must be defined for leaf nodes)
///
/// it is up to the implementation of Encoder to decide how the
/// game tree is navigated, in a neighbor-aware context. this Game
/// structure should only concern itself of local properties.
pub trait Game: Clone + Copy {
    type E: super::edge::Edge;
    type T: super::turn::Turn;
    fn root() -> Self;
    fn turn(&self) -> Self::T;
    fn apply(&self, edge: Self::E) -> Self;
    fn payoff(&self, turn: Self::T) -> crate::Utility;
}
