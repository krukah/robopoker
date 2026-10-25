use crate::*;

type MoveOf<T> = <<T as Approximation>::R as Rules>::Move;
type TurnOf<T> = <<T as Approximation>::R as Rules>::Turn;

/// An edge quotient over [`Rules`]: many exact moves coarsen to one edge, and every edge refines
/// back to one move. Law: `coarsen(r, past, refine(r, e)) == e`.
///
/// History stays in the tree, not in the state. `past` yields the path's abstract edges nearest
/// first, the order a [`Node`] walks upward in.
pub trait Approximation {
    type R: Rules;
    type E: CfrEdge;

    fn coarsen<P>(r: &Self::R, past: P, m: MoveOf<Self>) -> Self::E
    where
        P: IntoIterator<Item = Self::E>;

    /// At a chance edge, samples the outcome.
    fn refine(r: &Self::R, e: Self::E) -> MoveOf<Self>;

    /// The descents of an exact move list played from `root`.
    fn descents<I>(root: Self::R, moves: I) -> Vec<Descent<TurnOf<Self>, Self::E>>
    where
        I: IntoIterator<Item = MoveOf<Self>>,
    {
        moves
            .into_iter()
            .scan((root, Vec::new()), |(r, past), m| {
                let edge = Self::coarsen(r, past.iter().rev().copied(), m);
                let turn = r.turn();
                *r = r.apply(m);
                past.push(edge);
                Some(Descent(turn, edge))
            })
            .collect()
    }
}
