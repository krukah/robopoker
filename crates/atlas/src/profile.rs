//! Subgame profile that routes lookups between blueprint and local storage.
//!
//! During safe subgame solving, we maintain:
//! - A frozen blueprint profile for fallback reach/EV computation
//! - Fresh local regrets/weights for the subgame being solved
//!
//! The profile routes lookups based on whether local data exists,
//! falling through to clean blueprint values for unvisited infosets.
//!
//! World differentiation comes from the SubGameSolver's card restriction
//! mechanism (different worlds = different card deals = different game
//! trees), NOT from policy perturbation. This is the safe subgame
//! solving approach from Brown & Sandholm 2017.
use crate::*;
use pokerkit::*;
use regret::*;
use std::collections::HashMap;

/// Profile wrapper for safe subgame solving.
///
/// Routes strategy lookups between a frozen blueprint and fresh local storage.
/// When local data exists for a `(WorldInfo, Edge)` pair, returns it directly.
/// Otherwise falls through to the blueprint's accumulated values.
///
/// No perturbation is applied — worlds differentiate structurally through
/// the `WorldRestrict` mechanism which assigns different opponent cards per
/// world, producing distinct game trees and therefore distinct strategies.
pub struct WorldProfile<'blueprint, P>
where
    P: RefProf,
{
    global: &'blueprint P,
    local: HashMap<WorldInfo<P::I>, HashMap<P::E, Encounter>>,
    t: usize,
}

impl<'blueprint, P> WorldProfile<'blueprint, P>
where
    P: RefProf,
{
    pub fn new(blueprint: &'blueprint P) -> Self {
        Self {
            local: HashMap::new(),
            global: blueprint,
            t: 0,
        }
    }

    pub fn blueprint(&self) -> &P {
        self.global
    }
}

impl<P> CfrRule for WorldProfile<'_, P>
where
    P: RefProf,
{
    type T = P::T;
    type E = P::E;
    type G = P::G;
    type I = WorldInfo<P::I>;
}
impl<P> MutProf for WorldProfile<'_, P>
where
    P: RefProf,
{
    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut Probability {
        let blueprint = self.global;
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| blueprint.warmstart(&info.inner(), edge))
            .weight
    }

    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        let blueprint = self.global;
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| blueprint.warmstart(&info.inner(), edge))
            .regret
    }

    fn mut_payoff(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        let blueprint = self.global;
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| blueprint.warmstart(&info.inner(), edge))
            .payoff
    }

    fn mut_visits(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32 {
        let blueprint = self.global;
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| blueprint.warmstart(&info.inner(), edge))
            .visits
    }
}

impl<P> RefProf for WorldProfile<'_, P>
where
    P: RefProf,
{
    fn t(&self) -> usize {
        self.t
    }

    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.local
            .get(info)
            .and_then(|m| m.get(edge))
            .map_or_else(|| self.global.cum_weight(&info.inner(), edge).max(EPSILON), |e| e.weight)
    }

    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.local
            .get(info)
            .and_then(|m| m.get(edge))
            .map_or_else(|| self.global.cum_regret(&info.inner(), edge).max(EPSILON), |e| e.regret)
    }

    fn cum_payoff(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.local
            .get(info)
            .and_then(|m| m.get(edge))
            .map_or_else(|| self.global.cum_payoff(&info.inner(), edge), |e| e.payoff)
    }

    fn cum_visits(&self, info: &Self::I, edge: &Self::E) -> u32 {
        self.local
            .get(info)
            .and_then(|m| m.get(edge))
            .map_or_else(|| self.global.cum_visits(&info.inner(), edge), |e| e.visits)
    }

    fn sum_regret(&self) -> Utility {
        self.local
            .values()
            .flat_map(|edges| edges.values())
            .map(|e| e.regret.max(0.))
            .sum::<Utility>()
            / self.t.max(1) as Utility
    }
}

impl<P> CfrSampling for WorldProfile<'_, P>
where
    P: CfrSampling + RefProf,
{
    fn increment(&mut self) {
        self.t += 1;
    }

    fn walker(&self) -> Self::T {
        Self::T::from(self.t % Self::T::players())
    }

    fn temperature(&self) -> pokerkit::Entropy {
        self.global.temperature()
    }

    fn smoothing(&self) -> pokerkit::Energy {
        self.global.smoothing()
    }

    fn curiosity(&self) -> pokerkit::Probability {
        self.global.curiosity()
    }
}
