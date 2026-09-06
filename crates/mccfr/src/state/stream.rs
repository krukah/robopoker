//! Direction-aware projections over [`Descent`] streams.
//!
//! Three newtypes wrap `Vec<Descent<T, E>>`; they are structurally identical but
//! provenance makes them mean different things:
//!
//! - [`Replay`](crate::Replay) — full play-through from game root to "now",
//!   spanning chance transitions between streets.
//! - [`Prefix`](crate::Prefix) — immutable context fixed when a subgame solver
//!   was constructed. Never grows.
//! - [`Story`](crate::Story) — a growing rollout at a subgame frontier, kept
//!   distinct so the compiler refuses to splice it in as a prefix.
//!
//! All implement [`IntoIterator<Item = Descent<T, E>>`](IntoIterator), so
//! provenance-agnostic functions just accept that bound. `current_street` and
//! friends come from [`DescentStream`] when `T: CfrTurn` — non-turn anchors
//! (e.g. a [`Node`](crate::Node)) must map down to a turn first.
use crate::CfrEdge;
use crate::CfrTurn;
use crate::Descent;

/// Projections over any iterator of [`Descent`] pairs.
///
/// Everything here depends on the direction being downward from root, since
/// sequence order is what makes "trailing" and "current street" meaningful. For
/// an [`Ascent`](crate::Ascent) stream, collect + reverse + re-walk instead of
/// reinterpreting pairs in place: the fencepost shift across chance boundaries
/// is silent and wrong.
pub trait DescentStream<T, E>: IntoIterator<Item = Descent<T, E>> + Sized
where
    T: CfrTurn,
    E: CfrEdge,
{
    /// Trailing descents after the most recent chance boundary — a descent
    /// *anchored at* a chance turn, which is the street transition in NLHE.
    ///
    /// Empty if the sequence ends exactly at a chance descent; the full sequence
    /// if there are no chance anchors at all.
    fn current_street(self) -> Vec<E> {
        let walk: Vec<_> = self.into_iter().collect();
        let start = walk.iter().rposition(|d| d.0.is_chance()).map_or(0, |i| i + 1);
        walk[start..].iter().map(|d| d.1).collect()
    }
}

impl<S, T, E> DescentStream<T, E> for S
where
    S: IntoIterator<Item = Descent<T, E>>,
    T: CfrTurn,
    E: CfrEdge,
{
}

#[cfg(test)]
mod tests {
    use crate::*;
    use monge::Support;

    /// Minimal anchor turn for tests. Mirrors the three-variant shape of
    /// CfrTurn; we only exercise the chance/non-chance split here.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    enum T {
        Choice,
        Chance,
        Terminal,
    }
    impl From<usize> for T {
        fn from(_: usize) -> Self {
            T::Choice
        }
    }
    impl CfrTurn for T {
        fn chance() -> Self {
            T::Chance
        }
        fn terminal() -> Self {
            T::Terminal
        }
        fn players() -> usize {
            1
        }
    }

    /// Minimal test edge. `CfrEdge` defaults suffice; we just need Copy +
    /// the trait bound.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct E(u8);
    impl Support for E {}
    impl CfrEdge for E {}

    fn sample() -> Vec<Descent<T, E>> {
        vec![
            Descent(T::Choice, E(0)),
            Descent(T::Choice, E(1)),
            Descent(T::Chance, E(2)),
            Descent(T::Choice, E(3)),
            Descent(T::Choice, E(4)),
        ]
    }

    #[test]
    fn replay_trims_to_current_street() {
        let r = Replay::new(sample());
        assert_eq!(r.current_street(), vec![E(3), E(4)]);
    }

    #[test]
    fn replay_with_no_chance_returns_full() {
        let descents = vec![Descent(T::Choice, E(0)), Descent(T::Choice, E(1))];
        let r = Replay::new(descents);
        assert_eq!(r.current_street(), vec![E(0), E(1)]);
    }

    #[test]
    fn replay_ending_at_chance_has_empty_current() {
        let r = Replay::new(vec![Descent(T::Choice, E(0)), Descent(T::Chance, E(1))]);
        assert_eq!(r.current_street(), Vec::<E>::new());
    }

    #[test]
    fn prefix_and_story_use_same_trim() {
        let p = Prefix::new(sample());
        let t = Story::new(sample());
        assert_eq!(p.current_street(), t.current_street());
    }

    #[test]
    fn story_starts_from_prefix() {
        let prefix = Prefix::new(vec![Descent(T::Choice, E(0))]);
        let mut story = Story::from(&prefix);
        story.push(Descent(T::Choice, E(1)));
        assert_eq!(story.as_slice(), &[Descent(T::Choice, E(0)), Descent(T::Choice, E(1))]);
    }

    #[test]
    fn borrowed_iteration_preserves_order() {
        let r = Replay::new(sample());
        let collected: Vec<_> = (&r).into_iter().collect();
        assert_eq!(collected, r.clone().into_inner());
    }
}
