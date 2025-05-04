use super::edge::Edge;
use super::turn::Turn;
use crate::mccfr::traits::blueprint::Blueprint;
use crate::mccfr::traits::profile::Profile;
use std::collections::BTreeMap;

#[derive(Default)]
/// Blueprint represents the complete solution for the Rock Paper Scissors game,
/// storing accumulated regret and policy values that are built up during training.
/// It implements both Profile (for tracking historical regrets and policies) and
/// Trainer (for using those values to optimize strategy through counterfactual regret minimization).
/// As a Profile, it provides access to the current state. As a Trainer, it updates that state
/// to converge toward Nash equilibrium.
pub struct RPS {
    pub(super) epochs: usize,
    pub(super) encounters: BTreeMap<Turn, BTreeMap<Edge, (crate::Probability, crate::Utility)>>,
}

impl std::fmt::Display for RPS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Turns: {}", self.epochs)?;
        for (turn, edges) in &self.encounters {
            writeln!(f, "  {:?}:", turn)?;
            for (edge, _) in edges {
                writeln!(
                    f,
                    "    {:?}  R {:>+6.2}, W {:>6.2}, P {:>6.2},  A {:>6.2}",
                    edge,
                    self.profile().sum_regret(turn, edge),
                    self.profile().sum_policy(turn, edge),
                    self.profile().policy(turn, edge),
                    self.profile().advice(turn, edge),
                )?;
            }
        }
        Ok(())
    }
}
