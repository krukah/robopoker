//! Trait for computing biased continuation payoffs at frontier nodes.
use crate::*;
use mccfr::*;

/// Computes frontier payoff matrices. The implementor bundles a [`Encoding`] with a blueprint
/// profile, so `&self` serves both game encoding and strategy lookup.
///
/// `payoffs` is the only place the meaning of a `Continuation(k)` index is pinned down — that
/// contract is game-specific and lives nowhere else. A no-op continuation layer should return
/// `Payoffs::uniform(...)` explicitly.
pub trait DepthSampler<const D: usize>: Encoding {
    type Blueprint: Profile<F = Self::F>;
    fn blueprint(&self) -> &Self::Blueprint;
    /// Frontier EV matrix given the authoritative walk that led here.
    ///
    /// `prefix` carries `(turn, edge)` pairs captured by whoever walked
    /// the tree originally — the turns are ground truth, not
    /// reconstructed from edges (which is unsafe for games with
    /// chip-snapping or randomized chance).
    fn payoffs(&self, prefix: &Prefix<GameOf<Self>>, game: &GameOf<Self>, internal: TurnOf<Self>) -> Payoffs<D>;
}

/// A tabular solver samples frontiers with its encoder against its own profile.
impl<N, R, W, S, const B: usize, const D: usize> DepthSampler<D> for Tabular<N, R, W, S, B>
where
    N: Frontier<D>,
    R: RegretSchedule,
    W: WeightSchedule,
    S: SamplingScheme,
{
    type Blueprint = Book<N::F>;

    fn blueprint(&self) -> &Book<N::F> {
        self.profile()
    }

    fn payoffs(&self, prefix: &Prefix<GameOf<Self>>, game: &GameOf<Self>, internal: TurnOf<Self>) -> Payoffs<D> {
        self.encoder().payoffs(self.profile(), prefix, game, internal)
    }
}
