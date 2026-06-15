//! Direction-aware projections over [`Descent`] streams.
//!
//! The CFR tree is walked both downward from a root (via
//! [`CfrGame::apply`](crate::CfrGame::apply)) and upward from a leaf (via
//! tree ancestors). The in-memory representations of these walks are
//! structurally similar lists of [`Descent<T, E>`] pairs but mean different
//! things depending on where the walk came from:
//!
//! - [`Replay`](crate::Replay): the full play-through from game root to
//!   "now," spanning any chance-node transitions between streets.
//! - [`Prefix`](crate::Prefix): an immutable context fixed at
//!   subgame-solver construction time. Represents "these descents already
//!   happened before the solver even started." Never grows.
//! - [`Story`](crate::Story): a growing descent story emitted during
//!   a biased rollout at a subgame frontier. Distinct from the upstream
//!   solver's own internal path so the compiler refuses to splice it in
//!   as a prefix.
//!
//! All three carry `Vec<Descent<T, E>>` internally and implement
//! [`IntoIterator<Item = Descent<T, E>>`](IntoIterator). Functions that
//! don't care which of the three they get can simply accept
//! `impl IntoIterator<Item = Descent<T, E>>`. Direction-specific
//! projections (`current_street`) are available via the
//! [`DescentStream`] extension when `T: CfrTurn` — non-turn anchors
//! (e.g. a [`Node`](crate::Node) handle) should project down to a turn
//! with a mapping helper before reaching for these methods.
use crate::CfrEdge;
use crate::CfrTurn;
use crate::Descent;

/// Extension trait for any iterator of [`Descent`] pairs.
///
/// Everything here depends on direction being "downward from root" — the
/// sequence order defines what "trailing" and "current street" mean. For
/// an [`Ascent`](crate::Ascent) stream, collect + reverse + re-walk first
/// rather than trying to reinterpret the pairs in place (the fencepost
/// shift across chance boundaries is silent and wrong).
pub trait DescentStream<T, E>: IntoIterator<Item = Descent<T, E>> + Sized
where
    T: CfrTurn,
    E: CfrEdge,
{
    /// Trailing descents after the most recent chance boundary.
    ///
    /// "Chance boundary" here means a descent whose anchor turn is a
    /// chance node — i.e., an edge descended FROM a chance turn, which is
    /// the street transition for games like NLHE. If the sequence ends
    /// exactly at a chance descent, the result is empty. For games with
    /// no chance anchors at all, the result is the full sequence.
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
