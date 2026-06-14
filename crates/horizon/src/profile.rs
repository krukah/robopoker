//! Local mutable profile for depth-limited (leaf) solving.
use crate::*;
use pokerkit::*;
use regret::*;
use std::collections::HashMap;

pub struct DepthProfile<'blueprint, P, const D: usize>
where
    P: RefProf,
{
    /// Blueprint wrapped in a [`DepthView`] so that all `RefProf` calls
    /// (including [`RefProf::warmstart`]) dispatch correctly across
    /// `DepthEdge::Game` and `DepthEdge::Pick`. `Copy` because the view
    /// is just an `&P`; we copy it out of `&mut self` to dodge aliasing
    /// when `self.local` is mutably borrowed inside `or_insert_with`.
    view: DepthView<'blueprint, P, D>,
    local: HashMap<DepthInfo<P::I, D>, HashMap<DepthEdge<P::E, D>, Encounter>>,
    t: usize,
}

impl<'blueprint, P, const D: usize> DepthProfile<'blueprint, P, D>
where
    P: RefProf,
{
    pub fn new(blueprint: &'blueprint P) -> Self {
        Self {
            view: DepthView::new(blueprint),
            local: HashMap::new(),
            t: 0,
        }
    }
}

impl<P, const D: usize> CfrRule for DepthProfile<'_, P, D>
where
    P: RefProf,
{
    type T = P::T;
    type E = DepthEdge<P::E, D>;
    type G = DepthGame<P::G, D>;
    type I = DepthInfo<P::I, D>;
}

impl<P, const D: usize> MutProf for DepthProfile<'_, P, D>
where
    P: RefProf,
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

impl<P, const D: usize> RefProf for DepthProfile<'_, P, D>
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

impl<P, const D: usize> CfrSampling for DepthProfile<'_, P, D>
where
    P: RefProf + CfrSampling,
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
