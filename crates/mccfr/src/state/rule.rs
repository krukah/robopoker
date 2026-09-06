use super::*;

/// Bundles the four associated types defining an extensive-form game tree.
///
/// [`MutProf`](crate::MutProf) and [`RefProf`](crate::RefProf) both extend it,
/// unifying their associated types so `P::T` stays unambiguous with both in scope.
pub trait CfrRule: Sized {
    type T: CfrTurn;
    type E: CfrEdge;
    type G: CfrGame<E = Self::E, T = Self::T>;
    type I: CfrInfo<E = Self::E, T = Self::T>;
}
