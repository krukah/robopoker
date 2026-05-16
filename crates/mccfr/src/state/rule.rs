use super::*;

/// Base trait bundling the four associated types that define
/// an extensive-form game tree: turns, edges, games, and info sets.
///
/// Both [`Storage`](crate::Storage) and [`Profile`](crate::Profile)
/// extend this trait, ensuring their associated types are unified
/// and `P::T` is unambiguous when both are in scope.
pub trait CfrRule: Sized {
    type T: CfrTurn;
    type E: CfrEdge;
    type G: CfrGame<E = Self::E, T = Self::T>;
    type I: CfrInfo<E = Self::E, T = Self::T>;
}
