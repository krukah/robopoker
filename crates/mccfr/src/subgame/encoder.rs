//! Encoder wrapper for subgame-augmented games.
//!
//! Handles encoding for prefix, alternative-phase, and real-game-phase states.
//! Delegates info encoding to the inner encoder via `seed()`.
use super::*;
use crate::*;

/// Encoder wrapper for subgame-augmented games.
///
/// Handles encoding for all three phases:
/// - Prefix: replays forced history, returns single forced edge
/// - MetaGame: world selection at subgame root
/// - RealGame: normal subgame play
///
/// # Type Parameters
///
/// - `N`: Inner encoder type
///
/// # Subgame Context
///
/// The encoder stores a prefix history for replaying forced actions.
/// During prefix phase, only the forced edge from prefix[cursor] is
/// available. Info encoding delegates to the inner encoder's `seed()`.
pub struct SubEncoder<'blueprint, N>
where
    N: Encoder,
{
    /// Inner encoder for the base game.
    inner: &'blueprint N,
    /// History prefix to replay before subgame entry.
    prefix: Vec<N::E>,
}

impl<'blueprint, N> SubEncoder<'blueprint, N>
where
    N: Encoder,
{
    /// Creates a new subgame encoder with prefix history.
    pub fn new(inner: &'blueprint N, prefix: Vec<N::E>) -> Self {
        Self { inner, prefix }
    }
    /// Returns the inner encoder.
    pub fn encoder(&self) -> &N {
        self.inner
    }
    /// Returns the prefix history.
    pub fn prefix(&self) -> &[N::E] {
        &self.prefix
    }

    pub fn until(&self, i: usize) -> &[N::E] {
        &self.prefix[..i]
    }
    pub fn at(&self, i: usize) -> N::E {
        self.prefix[i]
    }
}

impl<N> Encoder for SubEncoder<'_, N>
where
    N: Encoder,
{
    type T = SubTurn<N::T>;
    type E = SubEdge<N::E>;
    type G = SubGame<N::G>;
    type I = SubInfo<N::I, N::E>;

    fn resume(&self, past: &[Self::E], game: &Self::G) -> Self::I {
        let inner = past
            .iter()
            .filter_map(|e| match e {
                SubEdge::Inner(e) => Some(*e),
                SubEdge::World(_) => None,
            })
            .collect::<Vec<_>>();
        SubInfo::Info(self.encoder().resume(&inner, &game.inner()))
    }

    fn seed(&self, game: &Self::G) -> Self::I {
        match game.phase() {
            SubPhase::Real(_) => unreachable!("seed() only called at root (Prefix or MetaGame)"),
            SubPhase::Meta => SubInfo::Root,
            SubPhase::Prefix(..) => SubInfo::Prefix(self.encoder().seed(&game.inner()), self.at(0)),
        }
    }

    fn info(
        &self,
        _: &Tree<Self::T, Self::E, Self::G, Self::I>,
        (_, game, _): Branch<Self::E, Self::G>,
    ) -> Self::I {
        match game.phase() {
            SubPhase::Meta => SubInfo::Root,
            SubPhase::Real(_) => SubInfo::Info(self.encoder().resume(self.prefix(), &game.inner())),
            SubPhase::Prefix(i, _) => SubInfo::Prefix(
                self.encoder().resume(self.until(i), &game.inner()),
                self.at(i),
            ),
        }
    }

    fn branches(
        &self,
        node: &Node<Self::T, Self::E, Self::G, Self::I>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        match node.game().phase() {
            SubPhase::Real(_) => node.branches(),
            SubPhase::Meta => node
                .game()
                .alternative_edges()
                .into_iter()
                .map(|e| (e, node.game().apply(e), node.index()))
                .collect(),
            SubPhase::Prefix(i, _) => {
                let edge = SubEdge::Inner(self.at(i));
                vec![(edge, node.game().apply(edge), node.index())]
            }
        }
    }
}
