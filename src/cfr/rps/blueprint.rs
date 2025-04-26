use super::edge::Edge;
use super::game::Game;
use super::turn::Turn;
use crate::cfr::traits::profile::Profile;
use crate::cfr::traits::trainer::Trainer;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Blueprint {
    epochs: usize,
    encounters: BTreeMap<Turn, BTreeMap<Edge, (crate::Probability, crate::Utility)>>,
}

impl Blueprint {
    pub fn at(&mut self, info: &Turn, edge: &Edge) -> &mut (crate::Probability, crate::Utility) {
        self.encounters
            .entry(info.clone())
            .or_insert_with(BTreeMap::default)
            .entry(edge.clone())
            .or_insert((0., 0.))
    }

    pub fn train() -> Self {
        let mut blueprint = Self::default();
        for i in 0..crate::CFR_ITERATIONS {
            for ref update in blueprint.batch() {
                blueprint.update_regret(update);
                blueprint.update_weight(update);
            }
            blueprint.epochs += 1;
            log::info!("{}", blueprint);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        blueprint
    }
}

impl Profile for Blueprint {
    type T = Turn;
    type E = Edge;
    type G = Game;
    type I = Turn;

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
        _: &crate::cfr::structs::node::Node<Self::T, Self::E, Self::G, Self::I>,
        branches: Vec<crate::cfr::types::branch::Branch<Self::E, Self::G>>,
    ) -> Vec<crate::cfr::types::branch::Branch<Self::E, Self::G>> {
        branches
    }
}

impl std::fmt::Display for Blueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Blueprint(epochs: {})", self.epochs)?;
        for (turn, edges) in &self.encounters {
            writeln!(f, "  {:?}:", turn)?;
            for (edge, (_, regret)) in edges {
                writeln!(
                    f,
                    "    {:?} -> (Policy: {:.2}, Regret: {:+.2})",
                    edge,
                    self.profile().policy(turn, edge),
                    regret
                )?;
            }
        }
        Ok(())
    }
}
