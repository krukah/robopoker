//! Local mutable profile for off-tree-augmented (nested) solving.
//!
//! Mirrors [`crate::DepthProfile`]: a local map of accumulators that falls
//! through to the wrapped blueprint (via [`NestView`]) on a miss. Off-tree
//! edges warmstart at [`Encounter::default`]; canonical edges inherit the
//! blueprint's trained values.
use super::*;
use mccfr::*;
use pokerkit::*;
use std::collections::HashMap;

type Off<P> = <<P as CfrRule>::G as Augmentable>::Off;

pub struct NestProfile<'blueprint, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    view: NestView<'blueprint, P>,
    local: HashMap<NestInfo<P::I, Off<P>>, HashMap<NestEdge<P::E, Off<P>>, Encounter>>,
    t: usize,
}

impl<'blueprint, P> NestProfile<'blueprint, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    pub fn new(blueprint: &'blueprint P) -> Self {
        Self {
            view: NestView::new(blueprint),
            local: HashMap::new(),
            t: 0,
        }
    }
}

impl<P> CfrRule for NestProfile<'_, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    type T = P::T;
    type E = NestEdge<P::E, Off<P>>;
    type G = NestGame<P::G>;
    type I = NestInfo<P::I, Off<P>>;
}

impl<P> MutProf for NestProfile<'_, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    fn mut_weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut Probability {
        let view = self.view;
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| view.warmstart(info, edge))
            .weight
    }

    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        let view = self.view;
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| view.warmstart(info, edge))
            .regret
    }

    fn mut_payoff(&mut self, info: &Self::I, edge: &Self::E) -> &mut Utility {
        let view = self.view;
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| view.warmstart(info, edge))
            .payoff
    }

    fn mut_visits(&mut self, info: &Self::I, edge: &Self::E) -> &mut u32 {
        let view = self.view;
        &mut self
            .local
            .entry(*info)
            .or_default()
            .entry(*edge)
            .or_insert_with(|| view.warmstart(info, edge))
            .visits
    }
}

impl<P> RefProf for NestProfile<'_, P>
where
    P: RefProf,
    P::G: Augmentable,
{
    fn t(&self) -> usize {
        self.t
    }

    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.local
            .get(info)
            .and_then(|m| m.get(edge))
            .map_or_else(|| self.view.cum_weight(info, edge).max(EPSILON), |e| e.weight)
    }

    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.local
            .get(info)
            .and_then(|m| m.get(edge))
            .map_or_else(|| self.view.cum_regret(info, edge).max(EPSILON), |e| e.regret)
    }

    fn cum_payoff(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.local
            .get(info)
            .and_then(|m| m.get(edge))
            .map_or_else(|| self.view.cum_payoff(info, edge), |e| e.payoff)
    }

    fn cum_visits(&self, info: &Self::I, edge: &Self::E) -> u32 {
        self.local
            .get(info)
            .and_then(|m| m.get(edge))
            .map_or_else(|| self.view.cum_visits(info, edge), |e| e.visits)
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

impl<P> CfrSampling for NestProfile<'_, P>
where
    P: RefProf + CfrSampling,
    P::G: Augmentable,
{
    fn increment(&mut self) {
        self.t += 1;
    }

    fn walker(&self) -> Self::T {
        Self::T::from(self.t % Self::T::players())
    }

    fn temperature(&self) -> Entropy {
        self.view.temperature()
    }

    fn smoothing(&self) -> Energy {
        self.view.smoothing()
    }

    fn curiosity(&self) -> Probability {
        self.view.curiosity()
    }
}
