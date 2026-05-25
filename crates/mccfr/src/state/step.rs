//! Directional (turn, edge) pairs for tree traversal.
//!
//! A tree edge connects a parent to a child. Either endpoint can serve as
//! the pair's "anchor turn"; which endpoint makes sense depends on the
//! traversal direction:
//!
//! - [`Descent`]: downward step from parent to child. The anchor is the
//!   parent — the turn you're AT when you choose to descend via this edge.
//! - [`Ascent`]: upward step from child to parent. The anchor is the parent
//!   again — the turn you ARRIVE at after ascending.
//!
//! Descent and Ascent carry the same underlying `(parent_turn, edge)` data
//! for any given tree edge. They are distinct types so the compiler can
//! enforce directional intent: a function accepting
//! `impl IntoIterator<Item = Descent<T, E>>` will not silently accept an
//! upward walk, and a naive rewrap would still land the sequence in the
//! wrong order. Converting between directions requires re-walking the tree
//! (or an explicit `collect` + `reverse` + re-flavor), which makes the
//! fencepost question an active design decision rather than a forgettable
//! one-liner.
//!
//! The [`Jump`] trait unifies the two for direction-agnostic operations
//! like "extract the edges." Direction-sensitive operations (e.g.
//! `current_street`, which splits on chance boundaries in the parent turn)
//! live only on Descent-flavored blankets, in sibling modules.
//!
//! # Generic over `T`
//!
//! `T` is deliberately unconstrained here — implementations can carry a
//! bare turn, a full game state, a [`Node`](crate::Node) handle, or
//! anything else that makes sense at the anchor point. Capability bounds
//! are introduced at the use site.
use crate::CfrEdge;

/// A downward step `(parent_turn, edge)` in the game tree.
///
/// The anchor turn is the origin of the descent — the node you're at
/// *before* following `edge` to its child.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Descent<T, E>(pub T, pub E);

/// An upward step `(edge, parent_turn)` in the game tree.
///
/// The anchor turn is the destination of the ascent — the node you arrive
/// at *after* following `edge` in reverse from its child.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ascent<E, T>(pub E, pub T);

/// Shared accessors for [`Descent`] and [`Ascent`].
///
/// The node slot is always whatever sits at the non-edge endpoint of the
/// pair; direction only decides which tuple position stores what. Blankets
/// that don't care about direction (edge extraction, counting by edge
/// predicate, etc.) can be written against `Jump` uniformly. Direction-
/// sensitive blankets should name `Descent` or `Ascent` explicitly.
pub trait Jump: Copy {
    type T: Copy;
    type E: Copy;
    fn node(self) -> Self::T;
    fn edge(self) -> Self::E;
}

impl<T, E> Jump for Descent<T, E>
where
    T: Copy,
    E: Copy,
{
    type T = T;
    type E = E;
    fn node(self) -> T {
        self.0
    }
    fn edge(self) -> E {
        self.1
    }
}

impl<E, T> Jump for Ascent<E, T>
where
    T: Copy,
    E: Copy,
{
    type T = T;
    type E = E;
    fn node(self) -> T {
        self.1
    }
    fn edge(self) -> E {
        self.0
    }
}

/// Direction-agnostic extensions available on any stream of [`Jump`]
/// pairs, regardless of `Descent` vs `Ascent` flavor.
///
/// Only edge-level projections live here. Anything that depends on the
/// structural meaning of the sequence order (e.g. splitting at chance
/// boundaries) must be keyed to a specific direction.
pub trait JumpStream: IntoIterator + Sized
where
    Self::Item: Jump,
{
    /// Discard the anchor turns; yield only edges in original order.
    fn edges(self) -> impl Iterator<Item = <Self::Item as Jump>::E> {
        self.into_iter().map(Jump::edge)
    }
    /// Count pairs whose edge satisfies `pred`.
    fn count_edges<F>(self, pred: F) -> usize
    where
        F: Fn(&<Self::Item as Jump>::E) -> bool,
    {
        self.into_iter().filter(|j| pred(&j.edge())).count()
    }
}

impl<S> JumpStream for S
where
    S: IntoIterator,
    S::Item: Jump,
{
}

/// Compose a downward walk from a root game state and a sequence of edges.
///
/// Folds `CfrGame::apply` over the edges, emitting `Descent(turn, edge)`
/// at each step where `turn` is the pre-apply turn at the origin. Works
/// for any game whose turn class is determined by round structure alone
/// (independent of chance entropy) — which is every `CfrGame` we care
/// about today. Intermediate game states are discarded; only turns are
/// observable. If you need full game reconstruction across chance
/// boundaries, the edge sequence is not enough — chance edges don't
/// carry the concrete outcome.
pub fn descents_from<G, I>(root: G, edges: I) -> impl Iterator<Item = Descent<G::T, G::E>>
where
    G: crate::CfrGame + Copy,
    G::T: Copy,
    G::E: CfrEdge,
    I: IntoIterator<Item = G::E>,
{
    edges.into_iter().scan(root, |g, e| {
        let turn = g.turn();
        *g = g.apply(e);
        Some(Descent(turn, e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_transport::Support;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct T(u8);
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct E(u8);
    impl Support for E {}
    impl CfrEdge for E {}

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

    #[test]
    fn count_edges_predicate() {
        let s = vec![Descent(T(0), E(1)), Descent(T(0), E(2)), Descent(T(0), E(3))];
        assert_eq!(s.count_edges(|e| e.0 > 1), 2);
    }
}
