//! RPS profile: stores accumulated strategies and regrets.
use super::*;
use crate::*;
use rbp_core::*;

impl<R, W, S, const N: usize> Profile for RPS<R, W, S, N>
where
    R: RegretSchedule,
    W: PolicySchedule,
    S: SamplingScheme,
{
    type T = RpsTurn;
    type E = RpsEdge;
    type G = RpsGame;
    type I = RpsInfo;
    fn increment(&mut self) {
        self.epochs += 1;
    }
    fn epochs(&self) -> usize {
        self.epochs
    }
    fn walker(&self) -> Self::T {
        match self.epochs() % 2 {
            0 => RpsTurn::P1,
            _ => RpsTurn::P2,
        }
    }
    fn cum_weight(&self, info: &Self::I, edge: &Self::E) -> Probability {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(w, _, _, _)| *w)
            .unwrap_or_default()
    }
    fn cum_regret(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, r, _, _)| *r)
            .unwrap_or_default()
    }
    fn cum_evalue(&self, info: &Self::I, edge: &Self::E) -> Utility {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, _, v, _)| *v)
            .unwrap_or_default()
    }
    fn cum_counts(&self, info: &Self::I, edge: &Self::E) -> u32 {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, _, _, c)| *c)
            .unwrap_or_default()
    }
    fn temperature(&self) -> Entropy {
        1.0
    }
    fn smoothing(&self) -> Energy {
        0.0
    }
    fn curiosity(&self) -> Probability {
        SAMPLING_CURIOSITY
    }
}
