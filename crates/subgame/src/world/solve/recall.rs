//! Bundled game state and action history at the subgame entry.
//!
//! Carries an authoritative `Descent<T, E>` stream — turns recorded at construction by whoever
//! actually walked the tree, not reconstructed later by re-applying edges. For games like NLHE,
//! re-applying is unsafe: chip snapping and chance-card randomness produce a different
//! trajectory than the original, and the divergence can silently change turn classes (e.g.
use mccfr::*;

/// The observed game state and action history at the subgame entry point.
///
/// `game` is the head, not a seed, so the states cannot be re-derived from it. It is a
/// [`Prefix`] paired with the position that prefix arrives at.
///
/// The descents provide action-with-anchor-turn context for [`Encoding::resume`] and any
/// downstream consumer that wants a [`Descent`] stream.
pub struct CfrRecall<G>
where
    G: CfrGame,
{
    descents: Prefix<G>,
    game: G,
}

impl<G> CfrRecall<G>
where
    G: CfrGame,
{
    /// Build a recall from authoritative `(turn, edge)` pairs plus the
    /// current game state. Caller is responsible for ensuring the
    /// descents faithfully describe the trajectory into `game`.
    pub fn new(descents: impl IntoIterator<Item = DescentOf<G>>, game: G) -> Self {
        Self {
            descents: descents.into_iter().collect(),
            game,
        }
    }

    /// The solver-preceding context, already typed as a [`Prefix`] — it
    /// never grows, which is exactly what that marker asserts.
    pub fn descents(&self) -> &Prefix<G> {
        &self.descents
    }

    pub fn game(&self) -> G {
        self.game
    }
}
