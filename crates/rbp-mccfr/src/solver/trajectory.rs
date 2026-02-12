//! Generic replay abstraction for applying edge sequences to game states.
//!
//! A [`Trajectory`] represents a root game state plus a sequence of edges taken.
//! This enables generic computation of decision points, reach probabilities,
//! and composition patterns like fan-in (same path, multiple roots) and
//! fan-out (same root, multiple paths).
//!
//! # Design
//!
//! `Trajectory` is pure data — it stores root + path but does not reference
//! encoder or profile. Those are passed to methods like [`Trajectory::internal_reach`]
//! when needed. This keeps Trajectory lightweight and composable.
//!
//! # NLHE Integration
//!
//! NLHE operates on `Action` at the gameplay layer but `Edge` at the CFR layer.
//! The conversion (`Action` → `Edge` via `Info::edgify`) happens at the boundary
//! when constructing a Trajectory from game history.
use super::waypoint::Waypoint;
use super::waypoints::Waypoints;
use crate::*;
use rbp_core::Probability;

/// A root game state plus sequence of edges taken.
///
/// Enables generic replay and decision extraction without building a full tree.
/// The path is stored as `Vec<E>` for genericity, though NLHE may pack into `Path`.
#[derive(Debug, Clone)]
pub struct Trajectory<G, E>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
{
    root: G,
    path: Vec<E>,
}

impl<G, E> From<(G, Vec<E>)> for Trajectory<G, E>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
{
    fn from((root, path): (G, Vec<E>)) -> Self {
        Self { root, path }
    }
}

impl<G, E> Trajectory<G, E>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
{
    /// The root game state.
    pub fn root(&self) -> G {
        self.root
    }
    /// The edge sequence.
    pub fn path(&self) -> &[E] {
        &self.path
    }
    /// Terminal game state after applying all edges.
    pub fn terminal(&self) -> G {
        self.path.iter().fold(self.root, |g, e| g.apply(*e))
    }
    /// All waypoints along the trajectory.
    ///
    /// Yields `Waypoint` for each edge in the path, with the game state
    /// at that point and the accumulated path up to (but not including)
    /// that edge.
    pub fn waypoints(&self) -> Waypoints<'_, G, E> {
        Waypoints {
            trajectory: self,
            state: self.root,
            index: 0,
        }
    }
    /// Decision points internal to a player's perspective.
    pub fn internal_decisions(&self, player: G::T) -> impl Iterator<Item = Waypoint<'_, G, E>> {
        self.waypoints().filter(move |w| w.state.turn() == player)
    }
    /// Decision points external to hero (not hero, not chance, not terminal).
    pub fn external_decisions(&self, hero: G::T) -> impl Iterator<Item = Waypoint<'_, G, E>> {
        self.waypoints()
            .filter(move |w| w.state.turn() != hero)
            .filter(|w| w.state.turn() != G::T::chance())
            .filter(|w| w.state.turn() != G::T::terminal())
    }
    /// Compute internal reach probability for a player along this trajectory.
    ///
    /// Products the averaged strategy probability at each of the player's
    /// decision points. Info is reconstructed via [`Encoder::resume`].
    pub fn internal_reach<N, P>(&self, player: G::T, encoder: &N, profile: &P) -> Probability
    where
        N: Encoder<T = G::T, E = E, G = G>,
        P: Profile<T = G::T, E = E, G = G, I = N::I>,
    {
        self.internal_decisions(player)
            .map(|w| (w.edge, encoder.resume(w.past, &w.state)))
            .map(|(e, i)| profile.averaged(&i, &e))
            .product()
    }
    /// Compute external reach probability along this trajectory.
    ///
    /// Info is reconstructed via [`Encoder::resume`].
    pub fn external_reach<N, P>(&self, hero: G::T, encoder: &N, profile: &P) -> Probability
    where
        N: Encoder<T = G::T, E = E, G = G>,
        P: Profile<T = G::T, E = E, G = G, I = N::I>,
    {
        self.external_decisions(hero)
            .map(|w| (w.edge, encoder.resume(w.past, &w.state)))
            .map(|(e, i)| profile.averaged(&i, &e))
            .product()
    }
}

impl<'path, G, E> IntoIterator for &'path Trajectory<G, E>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
{
    type Item = Waypoint<'path, G, E>;
    type IntoIter = Waypoints<'path, G, E>;
    fn into_iter(self) -> Self::IntoIter {
        self.waypoints()
    }
}

/// Sample: apply same path to multiple roots.
///
/// Useful for computing opponent range — iterate over all possible
/// opponent hands and compute reach for each.
pub fn sample<G, E, R, P>(roots: R, path: P) -> impl Iterator<Item = Trajectory<G, E>>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
    R: IntoIterator<Item = G>,
    P: IntoIterator<Item = E>,
{
    let path = path.into_iter().collect::<Vec<_>>();
    roots
        .into_iter()
        .map(move |root| Trajectory::from((root, path.clone())))
}

/// Branch: apply multiple paths to same root.
///
/// Useful for exploring continuations from a fixed position.
pub fn branch<G, E, P>(paths: P, root: G) -> impl Iterator<Item = Trajectory<G, E>>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
    P: IntoIterator<Item = Vec<E>>,
{
    paths
        .into_iter()
        .map(move |path| Trajectory::from((root, path)))
}

/// Cartesian product: all combinations of roots and paths.
pub fn product<G, E, R, P>(roots: R, paths: P) -> impl Iterator<Item = Trajectory<G, E>>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
    R: IntoIterator<Item = G>,
    P: IntoIterator<Item = Vec<E>> + Clone,
    R::IntoIter: Clone,
{
    roots.into_iter().flat_map(move |root| {
        paths
            .clone()
            .into_iter()
            .map(move |path| Trajectory::from((root, path)))
    })
}
