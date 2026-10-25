//! Direction-aware projections over [`Descent`] streams.
use crate::CfrEdge;
use crate::CfrTurn;
use crate::Descent;

/// Projections over any iterator of [`Descent`] pairs.
///
/// Everything here depends on the direction being downward from root, since sequence order is
/// what makes "trailing" and "current street" meaningful. For an [`Ascent`](crate::Ascent)
/// stream, collect + reverse + re-walk instead of reinterpreting pairs in place: the fencepost
/// shift across chance boundaries is silent and wrong.
pub trait Descents<T, E>
where
    Self: IntoIterator<Item = Descent<T, E>> + Sized,
    T: CfrTurn,
    E: CfrEdge,
{
    /// Trailing descents after the most recent chance boundary — a descent
    /// *anchored at* a chance turn, which is the street transition in NLHE.
    ///
    /// Empty if the sequence ends exactly at a chance descent; the full sequence
    /// if there are no chance anchors at all.
    fn current_street(self) -> Vec<E> {
        self.street().into_iter().map(|d| d.1).collect()
    }

    /// The descents [`Self::current_street`] keeps, turns included.
    fn street(self) -> Vec<Descent<T, E>> {
        let mut walk = self.into_iter().collect::<Vec<_>>();
        let start = walk.iter().rposition(|d| d.0.is_chance()).map_or(0, |i| i + 1);
        walk.split_off(start)
    }
}

impl<S, T, E> Descents<T, E> for S
where
    S: IntoIterator<Item = Descent<T, E>>,
    T: CfrTurn,
    E: CfrEdge,
{
}

#[cfg(test)]
mod tests {
    use crate::state::fixture::*;
    use crate::*;

    impl Descent<T, E> {
        fn sample() -> Vec<Self> {
            vec![
                Descent(T::Choice, E(0)),
                Descent(T::Choice, E(1)),
                Descent(T::Chance, E(2)),
                Descent(T::Choice, E(3)),
                Descent(T::Choice, E(4)),
            ]
        }
    }

    #[test]
    fn replay_trims_to_current_street() {
        let r = Replay::<G>::new(Descent::sample());
        assert_eq!(r.current_street(), vec![E(3), E(4)]);
    }

    #[test]
    fn replay_with_no_chance_returns_full() {
        let descents = vec![Descent(T::Choice, E(0)), Descent(T::Choice, E(1))];
        let r = Replay::<G>::new(descents);
        assert_eq!(r.current_street(), vec![E(0), E(1)]);
    }

    #[test]
    fn replay_ending_at_chance_has_empty_current() {
        let r = Replay::<G>::new(vec![Descent(T::Choice, E(0)), Descent(T::Chance, E(1))]);
        assert_eq!(r.current_street(), Vec::<E>::new());
    }

    #[test]
    fn prefix_and_story_use_same_trim() {
        let p = Prefix::<G>::new(Descent::sample());
        let t = Story::<G>::new(Descent::sample());
        assert_eq!(p.current_street(), t.current_street());
    }

    #[test]
    fn story_starts_from_prefix() {
        let prefix = Prefix::<G>::new(vec![Descent(T::Choice, E(0))]);
        let mut story = Story::from(&prefix);
        story.push(Descent(T::Choice, E(1)));
        assert_eq!(story.as_slice(), &[Descent(T::Choice, E(0)), Descent(T::Choice, E(1))]);
    }

    #[test]
    fn borrowed_iteration_preserves_order() {
        let r = Replay::<G>::new(Descent::sample());
        let collected: Vec<_> = (&r).into_iter().collect();
        assert_eq!(collected, r.clone().into_inner());
    }
}
