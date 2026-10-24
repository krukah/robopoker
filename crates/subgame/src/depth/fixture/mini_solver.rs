use super::*;
use crate::*;
use mccfr::*;

/// Bundles encoder + blueprint for depth-limited solving tests.
pub struct MiniSolver(pub MiniEncoder, pub MiniBlueprint);

impl CfrSpec for MiniSolver {
    type F = MiniInfo;
}

impl Encoding for MiniSolver {
    fn public<P>(&self, game: &GameOf<Self>, past: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        self.0.public(game, past)
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        self.0.secret(game);
    }
}

impl<const N: usize> DepthSampler<N> for MiniSolver {
    type Blueprint = MiniBlueprint;

    fn blueprint(&self) -> &MiniBlueprint {
        &self.1
    }

    fn payoffs(&self, prefix: &Prefix<GameOf<Self>>, game: &GameOf<Self>, _: TurnOf<Self>) -> Payoffs<N> {
        Payoffs::uniform(self.1.frontier_payoff(&self.encode(game, prefix.nearest())))
    }
}
