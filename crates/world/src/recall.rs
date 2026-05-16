//! Bundled game state and action history at the subgame entry.
//!
//! Carries an authoritative `Descent<T, E>` stream — turns recorded at
//! construction by whoever actually walked the tree, not reconstructed
//! later by re-applying edges. For games like NLHE, re-applying is
//! unsafe: chip snapping and chance-card randomness produce a different
//! trajectory than the original, and the divergence can silently change
//! turn classes (e.g. a snapped all-in landing the game in terminal
//! earlier). Deterministic games (Kuhn, Leduc, RPS, test toys) can feed
//! in descents via the [`descents_from`](rbp_mccfr::descents_from)
//! helper; games that aren't guaranteed-deterministic must supply their
//! own turn-faithful descents.
use rbp_mccfr::*;

/// The observed game state and action history at the subgame entry point.
///
/// The descents provide action-with-anchor-turn context for
/// [`CfrEncoder::resume`] and any downstream consumer that wants a
/// [`Descent`] stream. The game provides the actual state for
/// [`Restricted::restrict`]. These must describe the same position — the
/// game is the result of applying the descents' edges from root.
pub struct CfrRecall<G>
where
    G: CfrGame,
{
    descents: Vec<Descent<G::T, G::E>>,
    game: G,
}

impl<G> CfrRecall<G>
where
    G: CfrGame,
{
    /// Build a recall from authoritative `(turn, edge)` pairs plus the
    /// current game state. Caller is responsible for ensuring the
    /// descents faithfully describe the trajectory into `game`.
    pub fn new(
        descents: impl IntoIterator<Item = Descent<G::T, G::E>>,
        game: G,
    ) -> Self {
        Self {
            descents: descents.into_iter().collect(),
            game,
        }
    }

    pub fn descents(&self) -> &[Descent<G::T, G::E>] {
        &self.descents
    }

    pub fn game(&self) -> G {
        self.game
    }
}
