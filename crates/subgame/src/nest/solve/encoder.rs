//! Encoding for off-tree-augmented (nested) subgames.
//!
//! Wraps a base [`Encoding`] whose game is [`Augmentable`] and lifts its info into the three-
//! way [`NestInfo`].
//!
//! ([`NestGame::is_entry`]), not the path.
//!
//! canonical descendants tag `Game`, keeping their regrets separate.
use crate::*;
use mccfr::*;

type Off<N> = <GameOf<N> as Augmentable>::Off;

impl<N> CfrSpec for NestEncoder<'_, N>
where
    N: Encoding,
    GameOf<N>: Augmentable,
{
    type F = NestInfo<InfoOf<N>, Off<N>>;
}

impl<N> Encoding for NestEncoder<'_, N>
where
    N: Encoding,
    GameOf<N>: Augmentable,
{
    /// The inner public half from the path without its off-tree marker, then the three-way tag.
    fn public<P>(&self, game: &GameOf<Self>, past: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        let path = past.into_iter().collect::<Vec<_>>();
        let sawoff = path.iter().any(|e| e.offtree().is_some());
        NestPublic::tag(
            self.inner()
                .public(game.inner(), path.into_iter().filter_map(NestEdge::game)),
            game,
            sawoff,
        )
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        self.inner().secret(game.inner())
    }
}
