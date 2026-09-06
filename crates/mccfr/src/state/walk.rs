//! Linear game histories: a seed plus a lossless move list.
use crate::*;
use pokerkit::Translated;

/// A linear game history. Everything beyond the seed and the move list is
/// derived by folding.
///
/// This is `kicker::Recall` generalized: `Game` becomes [`Self::G`],
/// `Action` becomes [`Self::Exact`], and `Edge` becomes `G::E`. The two
/// associated types are the whole point — `Exact` is what *happened*,
/// `G::E` is what the abstraction *saw*.
///
/// Every projection takes the [`Lens`] as a parameter rather than owning
/// one, because a single history must be re-coarsened under different
/// policies: a nearest-grid snap for blueprint lookup, an exact pass for
/// off-tree nesting. Walks own data; lenses own policy.
///
/// Each projection drives the lens exactly once per move, so a stateful
/// lens stays consistent within one call. Use a fresh lens per projection.
pub trait Walk {
    /// The game whose states this history walks.
    type G: CfrGame;

    /// The lossless move type — see [`Lens::Exact`].
    type Exact: Copy;

    /// The seed state the moves replay from.
    fn root(&self) -> Self::G;

    /// The move sequence, in order.
    fn moves(&self) -> &[Self::Exact];

    /// States from root to head — one per prefix, so `moves().len() + 1`
    /// entries with the root first and the head last.
    fn states<L>(&self, lens: &mut L) -> Vec<Self::G>
    where
        L: Lens<Self::G, Exact = Self::Exact>,
    {
        std::iter::once(self.root())
            .chain(self.moves().iter().scan(self.root(), |state, exact| {
                *state = lens.apply(state, *exact);
                Some(*state)
            }))
            .collect()
    }

    /// The state after every move has been applied.
    fn head<L>(&self, lens: &mut L) -> Self::G
    where
        L: Lens<Self::G, Exact = Self::Exact>,
    {
        self.moves()
            .iter()
            .fold(self.root(), |state, exact| lens.apply(&state, *exact))
    }

    /// The coarsened history, preserving moves the lens declined to snap.
    /// The total, honest projection — mirrors `Recall::typed_history`.
    fn typed<L>(&self, lens: &mut L) -> Vec<Translated<<Self::G as CfrGame>::E, Self::Exact>>
    where
        L: Lens<Self::G, Exact = Self::Exact>,
    {
        self.moves()
            .iter()
            .scan(self.root(), |state, exact| {
                let seen = lens.coarsen(state, *exact);
                *state = lens.apply(state, *exact);
                Some(seen)
            })
            .collect()
    }

    /// The coarsened history as bare edges. Panics if the lens declines to
    /// snap — mirrors `Recall::history`. Reach for [`Self::typed`] when
    /// off-tree moves are possible.
    fn edges<L>(&self, lens: &mut L) -> Vec<<Self::G as CfrGame>::E>
    where
        L: Lens<Self::G, Exact = Self::Exact>,
    {
        self.typed(lens)
            .into_iter()
            .map(|snapped| match snapped {
                Translated::Snap(edge) => edge,
                Translated::Free(_) => unreachable!("off-tree move under a snapping lens"),
            })
            .collect()
    }

    /// The walk materialized as a [`Replay`] — the bridge into the
    /// [`DescentStream`] vocabulary, so `current_street` and friends apply.
    fn descents<L>(&self, lens: &mut L) -> Replay<<Self::G as CfrGame>::T, <Self::G as CfrGame>::E>
    where
        L: Lens<Self::G, Exact = Self::Exact>,
    {
        self.moves()
            .iter()
            .scan(self.root(), |state, exact| {
                let jump = Descent(state.turn(), lens.coarsen(state, *exact));
                *state = lens.apply(state, *exact);
                Some(jump)
            })
            .map(|Descent(turn, snapped)| match snapped {
                Translated::Snap(edge) => Descent(turn, edge),
                Translated::Free(_) => unreachable!("off-tree move under a snapping lens"),
            })
            .collect()
    }
}
