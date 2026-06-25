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
    /// If true, current-street chance nodes become continuation-choice frontiers.
    depth_limited: bool,
}

impl<'blueprint, N> SubEncoder<'blueprint, N>
where
    N: Encoder,
{
    /// Creates a new subgame encoder with prefix history.
    pub fn new(inner: &'blueprint N, prefix: Vec<N::E>) -> Self {
        Self {
            inner,
            prefix,
            depth_limited: false,
        }
    }
    /// Creates a depth-limited encoder that stops before chance/street transitions.
    pub fn depth_limited(inner: &'blueprint N, prefix: Vec<N::E>) -> Self {
        // The prefix is the already-observed current-round line. Replaying it
        // keeps the actual hand consistent before the frontier search branches.
        Self {
            inner,
            prefix,
            depth_limited: true,
        }
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
    fn inner_path(
        &self,
        tree: &Tree<SubTurn<N::T>, SubEdge<N::E>, SubGame<N::G>, SubInfo<N::I, N::E>>,
        (edge, _, head): &Branch<SubEdge<N::E>, SubGame<N::G>>,
    ) -> Vec<N::E> {
        std::iter::once(*edge)
            .chain(tree.at(*head).map(|(_, e)| e))
            .filter_map(|edge| match edge {
                SubEdge::Inner(edge) => Some(edge),
                SubEdge::World(_) | SubEdge::Continuation(_) => None,
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
    fn dls_child(
        &self,
        edge: SubEdge<N::E>,
        game: SubGame<N::G>,
        parent: petgraph::graph::NodeIndex,
    ) -> Branch<SubEdge<N::E>, SubGame<N::G>> {
        let game = if self.depth_limited && game.is_real_chance() {
            game.frontier()
        } else {
            game
        };
        (edge, game, parent)
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
                SubEdge::World(_) | SubEdge::Continuation(_) => None,
            })
            .collect::<Vec<_>>();
        SubInfo::Info(self.encoder().resume(&inner, &game.inner()))
    }

    fn seed(&self, game: &Self::G) -> Self::I {
        match game.phase() {
            SubPhase::Real(_) | SubPhase::Frontier(_) | SubPhase::Terminal(_, _) => {
                unreachable!("seed() only called at root (Prefix or MetaGame)")
            }
            SubPhase::Meta => SubInfo::Root,
            SubPhase::Prefix(..) => SubInfo::Prefix(self.encoder().seed(&game.inner()), self.at(0)),
        }
    }

    fn info(
        &self,
        tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        branch @ (_, game, _): Branch<Self::E, Self::G>,
    ) -> Self::I {
        match game.phase() {
            SubPhase::Meta => SubInfo::Root,
            SubPhase::Real(_) => SubInfo::Info(
                self.encoder()
                    .resume(&self.inner_path(tree, &branch), &game.inner()),
            ),
            SubPhase::Frontier(_) => SubInfo::Frontier(
                self.encoder()
                    .resume(&self.inner_path(tree, &branch), &game.inner()),
            ),
            SubPhase::Terminal(_, _) => SubInfo::Frontier(
                self.encoder()
                    .resume(&self.inner_path(tree, &branch), &game.inner()),
            ),
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
            SubPhase::Real(_) => node
                .branches()
                .into_iter()
                .map(|(edge, game, parent)| self.dls_child(edge, game, parent))
                .collect(),
            SubPhase::Frontier(_) => node
                .game()
                .continuation_edges()
                .into_iter()
                .map(|e| (e, node.game().apply(e), node.index()))
                .collect(),
            SubPhase::Terminal(_, _) => vec![],
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
