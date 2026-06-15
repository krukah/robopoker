//! Trait for computing biased continuation payoffs at frontier nodes.
use crate::*;
use mccfr::*;

/// Trait for computing frontier payoff matrices.
///
/// The implementor bundles a [`CfrEncoder`] with a blueprint profile,
/// so `&self` provides both game encoding and strategy lookup.
/// The const generic D determines the number of continuation strategies.
///
/// Implementors must define `payoffs` — the semantic meaning of each
/// `Continuation(k)` index is game-specific, and producing a D×D matrix
/// of EVs is the only place that contract is encoded. For a no-op
/// continuation layer, return `Payoffs::uniform(...)` explicitly.
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
