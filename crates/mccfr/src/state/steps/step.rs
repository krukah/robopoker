//! Directional (turn, edge) pairs for tree traversal: [`Descent`] steps parent → child,
//! [`Ascent`] steps child → parent. Both anchor on the *parent* turn and so carry identical
//! data for a given tree edge.

use super::*;
/// A downward step `(parent_turn, edge)`: the anchor is where you are *before*
/// following `edge` to its child.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Descent<T, E>(pub T, pub E);

/// A game's own downward step: its turn, then its edge.
pub type DescentOf<G> = Descent<<G as crate::CfrGame>::T, <G as crate::CfrGame>::E>;

/// An upward step `(edge, parent_turn)`: the anchor is where you arrive *after*
/// following `edge` in reverse from its child.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ascent<E, T>(pub E, pub T);

/// Shared accessors for [`Descent`] and [`Ascent`].
///
/// The node slot is whatever sits at the non-edge endpoint; direction only
/// decides which tuple position holds it. Direction-sensitive blankets should
/// name `Descent` or `Ascent` explicitly rather than bound on `Jump`.
pub trait Jump: Copy {
    type N: Copy;
    type E: Copy;
    fn node(self) -> Self::N;
    fn edge(self) -> Self::E;
}

impl<N, E> Jump for Descent<N, E>
where
    N: Copy,
    E: Copy,
{
    type N = N;
    type E = E;
    fn node(self) -> N {
        self.0
    }
    fn edge(self) -> E {
        self.1
    }
}

impl<E, N> Jump for Ascent<E, N>
where
    N: Copy,
    E: Copy,
{
    type N = N;
    type E = E;
    fn node(self) -> N {
        self.1
    }
    fn edge(self) -> E {
        self.0
    }
}

impl<S> Jumps for S
where
    S: IntoIterator,
    S::Item: Jump,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::fixture::E;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct T(u8);

    #[test]
    fn descent_accessors() {
        let d = Descent(T(7), E(1));
        assert_eq!(d.node(), T(7));
        assert_eq!(d.edge(), E(1));
    }

    #[test]
    fn ascent_accessors() {
        let a = Ascent(E(1), T(7));
        assert_eq!(a.node(), T(7));
        assert_eq!(a.edge(), E(1));
    }

    #[test]
    fn edges_projection_direction_blind() {
        let descents = vec![Descent(T(0), E(1)), Descent(T(1), E(2))];
        let ascents = vec![Ascent(E(1), T(0)), Ascent(E(2), T(1))];
        let from_desc: Vec<E> = descents.edges().collect();
        let from_asc: Vec<E> = ascents.edges().collect();
        assert_eq!(from_desc, vec![E(1), E(2)]);
        assert_eq!(from_asc, vec![E(1), E(2)]);
    }
}
