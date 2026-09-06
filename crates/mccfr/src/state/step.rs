//! Directional (turn, edge) pairs for tree traversal: [`Descent`] steps parent →
//! child, [`Ascent`] steps child → parent. Both anchor on the *parent* turn and
//! so carry identical data for a given tree edge.
//!
//! They are distinct types so the compiler enforces directional intent: a fn
//! taking `impl IntoIterator<Item = Descent<T, E>>` won't silently accept an
//! upward walk, and a naive rewrap would still leave the sequence reversed.
//! Converting directions requires re-walking (or explicit `collect` + `reverse` +
//! re-flavor), which makes the fencepost an active decision rather than a
//! forgettable one-liner.
//!
//! [`Jump`] unifies the two for direction-agnostic work like edge extraction.
//! Direction-sensitive operations (e.g. `current_street`, which splits on chance
//! boundaries in the parent turn) live only on Descent-flavored blankets.
//!
//! `T` is deliberately unconstrained — a bare turn, a full game state, a
//! [`Node`](crate::Node) handle. Capability bounds arrive at the use site.
use crate::CfrEdge;

/// A downward step `(parent_turn, edge)`: the anchor is where you are *before*
/// following `edge` to its child.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Descent<T, E>(pub T, pub E);

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

/// Direction-agnostic extensions on any stream of [`Jump`] pairs.
///
/// Only edge-level projections belong here; anything depending on the structural
/// meaning of sequence order (e.g. splitting at chance boundaries) must be keyed
/// to a specific direction.
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

/// Compose a downward walk from a root game state and a sequence of edges,
/// folding `CfrGame::apply` and emitting the pre-apply turn at each step.
///
/// Valid only because turn class is determined by round structure alone,
/// independent of chance entropy — true of every `CfrGame` today. Intermediate
/// game states are discarded, and edges alone cannot reconstruct them across
/// chance boundaries since chance edges don't carry the concrete outcome.
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
    use monge::Support;

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
