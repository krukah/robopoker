use crate::*;
use mccfr::*;

/// How an encoder values a depth-limited frontier against a blueprint. The default, uniform
/// payoffs at the blueprint's frontier value, ignores which continuation each player picks.
pub trait Frontier<const D: usize>: Encoding {
    fn payoffs<P>(
        &self,
        blueprint: &P,
        prefix: &Prefix<GameOf<Self>>,
        game: &GameOf<Self>,
        internal: TurnOf<Self>,
    ) -> Payoffs<D>
    where
        P: Profile<F = Self::F>,
    {
        let _ = internal;
        Payoffs::uniform(blueprint.frontier_payoff(&self.encode(game, prefix.nearest())))
    }
}
