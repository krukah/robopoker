//! Transcoding between a game's lossless move space and its abstract edges.
use crate::*;
use pokerkit::Translated;

/// Transcoder between a game's lossless move space and its abstract edges.
///
/// The two spaces differ along one axis that covers *both* bet sizing and
/// chance. For NLHE:
///
/// | | lossless `Exact` | lossy `G::E` |
/// |---|---|---|
/// | choice | `Raise(Chips)` | `Raise(Odds)` |
/// | chance | `Draw(Hand)` | `Draw` |
///
/// A lens is **stateful and order-sensitive**: it observes every move of a
/// walk in sequence, so it can accumulate running context (trailing
/// aggression, for bet-size translation), advance an RNG, or pop the next
/// recorded deal. Each projection on [`Walk`] drives the lens exactly once
/// per move — do not reuse a lens across two projections of the same walk.
///
/// The directions are not symmetric. [`Self::coarsen`] is total and
/// forgetful. [`Self::refine`] is a *section*: it must invent the detail
/// coarsening threw away. At choice edges the game supplies a canonical
/// answer (nearest legal chip amount); at chance edges the invention is
/// **destiny** — an RNG, a recorded deal, or a script.
///
/// **Law:** `coarsen(s, refine(s, e)) == Snap(e)`. The reverse does not
/// hold, and that asymmetry is exactly the abstraction's loss.
pub trait Lens<G>
where
    G: CfrGame,
{
    /// The lossless move: chip-exact and chance-resolved. NLHE: `kicker::Action`.
    type Exact: Copy;

    /// The lossless transition.
    ///
    /// Deliberately distinct from [`CfrGame::apply`], which consumes a
    /// lossy edge and — for games with chance — resolves the deal at
    /// random. Routing transitions through the lens is what lets a
    /// scripted lens reproduce a recorded hand exactly.
    fn apply(&mut self, state: &G, exact: Self::Exact) -> G;

    /// Total and lossy: forget chip precision and chance outcomes.
    ///
    /// May decline to snap, handing the move back as [`Translated::Free`] —
    /// an off-grid raise under an abstraction-free translation policy.
    fn coarsen(&mut self, state: &G, exact: Self::Exact) -> Translated<G::E, Self::Exact>;

    /// A section of [`Self::coarsen`] — realize an edge as a concrete move.
    fn refine(&mut self, state: &G, edge: G::E) -> Self::Exact;
}
