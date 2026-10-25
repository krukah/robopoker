use super::*;
use mccfr::*;
use pokerkit::*;

pub struct MiniBlueprint;

impl CfrSpec for MiniBlueprint {
    type F = MiniInfo;
}

impl Profile for MiniBlueprint {
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
