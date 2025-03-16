use super::edge::Edge;
use super::game::Game;
use super::head::Head;

/// Represents a leaf node in the game tree
pub trait Leaf: Clone + Copy {
    fn head<T>(&self) -> &T
    where
        T: Head;

    fn edge<T>(&self) -> &T
    where
        T: Edge;

    fn game<T>(&self) -> &T
    where
        T: Game;
}
