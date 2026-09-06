//! The trivial lens, for walks already in abstract space.
use crate::*;
use pokerkit::Translated;

/// The trivial lens: the walk's moves *are* the abstract edges, so both
/// directions are the identity and nothing is lost — or rather, nothing is
/// lost *here*, because it was already lost upstream.
///
/// [`Lens::apply`] delegates to [`CfrGame::apply`], which resolves chance
/// at **random** for games like NLHE. So states reconstructed through
/// `Ident` across a chance boundary are not the states originally walked.
/// This is the type-level form of the caveat on
/// [`descents_from`](crate::descents_from): an edge sequence alone cannot
/// reproduce a hand. Use a scripted lens when the deals must be replayed.
pub struct Ident<G>(std::marker::PhantomData<G>)
where
    G: CfrGame;

impl<G> Default for Ident<G>
where
    G: CfrGame,
{
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<G> Lens<G> for Ident<G>
where
    G: CfrGame,
{
    type Exact = G::E;

    fn apply(&mut self, state: &G, exact: G::E) -> G {
        state.apply(exact)
    }

    fn coarsen(&mut self, _: &G, exact: G::E) -> Translated<G::E, G::E> {
        Translated::Snap(exact)
    }

    fn refine(&mut self, _: &G, edge: G::E) -> G::E {
        edge
    }
}
