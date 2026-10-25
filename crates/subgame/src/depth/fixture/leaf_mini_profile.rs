use super::*;
use crate::*;
use mccfr::*;
use pokerkit::*;

pub struct LeafMiniProfile<const D: usize>;

impl<const D: usize> CfrSpec for LeafMiniProfile<D> {
    type F = DepthInfo<MiniInfo, D>;
}

impl<const D: usize> Profile for LeafMiniProfile<D> {
    fn t(&self) -> usize {
        1
    }

    fn sum_regret(&self) -> Utility {
        0.0
    }

    fn cum_weight(&self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> Probability {
        1.0
    }

    fn cum_regret(&self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> Utility {
        1.0
    }

    fn cum_payoff(&self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> Utility {
        0.5
    }

    fn cum_visits(&self, _: &InfoOf<Self>, _: &EdgeOf<Self>) -> u32 {
        1
    }
}

impl<const D: usize> Clock for LeafMiniProfile<D> {
    fn increment(&mut self) {}

    fn walker(&self) -> TurnOf<Self> {
        MiniTurn::P0
    }
}
