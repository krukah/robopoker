use super::edge::Edge;
use super::game::Game;
use super::rules::Rules;
use super::turn::Turn;
use crate::cfr::structs::node::Node;
use crate::cfr::traits::profile::Profile;
use crate::cfr::traits::trainer::Trainer;
use crate::cfr::types::branch::Branch;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Blueprint {
    epochs: usize,
    encounters: BTreeMap<Turn, BTreeMap<Edge, (crate::Probability, crate::Utility)>>,
}

impl Blueprint {
    pub fn train() -> Self {
        let mut blueprint = Self::default();
        // use crate::cfr::traits::trainer::Trainer;
        // blueprint.solve();
        // blueprint
        for i in 0..crate::CFR_ITERATIONS {
            log::trace!("training iteration {}", i);
            for ref update in blueprint.batch() {
                blueprint.update_regret(update);
                blueprint.update_weight(update);
            }
            log::info!("{}", blueprint);
            Profile::increment(&mut blueprint);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        blueprint
    }

    pub fn at(&mut self, info: &Turn, edge: &Edge) -> &mut (crate::Probability, crate::Utility) {
        self.encounters
            .entry(info.clone())
            .or_insert_with(BTreeMap::default)
            .entry(edge.clone())
            .or_insert((0., 0.))
    }
}

impl Trainer for Blueprint {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;
    type P = Blueprint;
    type S = Rules;

    fn encoder(&self) -> &Self::S {
        &Rules
    }

    fn profile(&self) -> &Self::P {
        self
    }

    fn discount(&self, _: Option<crate::Utility>) -> f32 {
        1.
    }

    fn regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.at(info, edge).1
    }

    fn weight(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self.at(info, edge).0
    }

    fn increment(&mut self) {
        Profile::increment(self);
    }
}

impl Profile for Blueprint {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;

    fn increment(&mut self) {
        self.epochs += 1;
    }

    fn walker(&self) -> Self::T {
        match self.epochs % 2 {
            0 => Turn::P1,
            _ => Turn::P2,
        }
    }

    fn epochs(&self) -> usize {
        self.epochs
    }

    fn weight(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        self.encounters
            .get(info)
            .and_then(|encounters| encounters.get(edge))
            .map(|(w, _)| *w)
            .unwrap_or(0.)
    }

    fn regret(&self, info: &Self::I, edge: &Self::E) -> crate::Utility {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, r)| *r)
            .unwrap_or(0.0)
    }

    fn sample(
        &self,
        _: &Node<Self::T, Self::E, Self::G, Self::I>,
        branches: Vec<Branch<Self::E, Self::G>>,
    ) -> Vec<Branch<Self::E, Self::G>> {
        branches
    }
}

impl std::fmt::Display for Blueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Turns: {}", self.epochs)?;
        for (turn, edges) in &self.encounters {
            writeln!(f, "  {:?}:", turn)?;
            for (edge, _) in edges {
                writeln!(
                    f,
                    "    {:?} -> W: {:.2}, P: {:.2}, A: {:.2}, R: {:+.2}",
                    edge,
                    self.profile().weight(turn, edge),
                    self.profile().policy(turn, edge),
                    self.profile().advice(turn, edge),
                    self.profile().regret(turn, edge)
                )?;
            }
        }
        Ok(())
    }
}
