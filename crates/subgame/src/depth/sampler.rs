//! Trait for computing biased continuation payoffs at frontier nodes.
use super::*;
use mccfr::*;

/// Computes frontier payoff matrices. The implementor bundles a
/// [`CfrEncoder`] with a blueprint profile, so `&self` serves both game
/// encoding and strategy lookup.
///
/// `payoffs` is the only place the meaning of a `Continuation(k)` index is
/// pinned down — that contract is game-specific and lives nowhere else. A
/// no-op continuation layer should return `Payoffs::uniform(...)` explicitly.
pub trait DepthSampler<const D: usize>: CfrEncoder {
    type Blueprint: RefProf<T = Self::T, E = Self::E, G = Self::G, I = Self::I>;
    fn blueprint(&self) -> &Self::Blueprint;
    /// Frontier EV matrix given the authoritative walk that led here.
    ///
    /// `prefix` carries `(turn, edge)` pairs captured by whoever walked
    /// the tree originally — the turns are ground truth, not
    /// reconstructed from edges (which is unsafe for games with
    /// chip-snapping or randomized chance).
    fn payoffs(&self, prefix: &Prefix<Self::T, Self::E>, game: &Self::G, internal: Self::T) -> Payoffs<D>;
}
