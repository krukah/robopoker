use super::edge::Edge;
use super::game::Game;
use super::turn::Turn;
use crate::cfr::rps::blueprint::Blueprint;
use crate::cfr::structs::node::Node;
use crate::cfr::traits::profile::Profile;
use crate::cfr::types::branch::Branch;

/// For the Rock Paper Scissors game, Blueprint implements the Profile trait.
/// It tracks regrets and policies over time.
impl Profile for Blueprint {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;

    fn increment(&mut self) {
        self.epochs += 1;
    }

    fn epochs(&self) -> usize {
        self.epochs
    }

    fn walker(&self) -> Self::T {
        match self.epochs() % 2 {
            0 => Turn::P1,
            _ => Turn::P2,
        }
    }

    fn weight(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(w, _)| *w)
            .unwrap_or_default()
    }

    fn regret(&self, info: &Self::I, edge: &Self::E) -> crate::Utility {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, r)| *r)
            .unwrap_or_default()
    }

    fn sample(
        &self,
        _: &Node<Self::T, Self::E, Self::G, Self::I>,
        branches: Vec<Branch<Self::E, Self::G>>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        branches
    }
}
