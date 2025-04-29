use super::edge::Edge;
use super::turn::Turn;
use crate::cfr::traits::profile::Profile;
use crate::cfr::traits::trainer::Trainer;
use std::collections::BTreeMap;

#[derive(Default)]
/// Blueprint represents the complete solution for the Rock Paper Scissors game,
/// storing accumulated regret and policy values that are built up during training.
/// It implements both Profile (for tracking historical regrets and policies) and
/// Trainer (for using those values to optimize strategy through counterfactual regret minimization).
/// As a Profile, it provides access to the current state. As a Trainer, it updates that state
/// to converge toward Nash equilibrium.
pub struct Blueprint {
    pub(super) epochs: usize,
    pub(super) encounters: BTreeMap<Turn, BTreeMap<Edge, (crate::Probability, crate::Utility)>>,
}

impl Blueprint {
    pub fn train() -> Self {
        let solution = Self::default().solve();
        log::info!("{}", solution);
        solution
    }

    pub fn at(&mut self, info: &Turn, edge: &Edge) -> &mut (crate::Probability, crate::Utility) {
        self.encounters
            .entry(info.clone())
            .or_insert_with(BTreeMap::default)
            .entry(edge.clone())
            .or_insert((0., 0.))
    }

    pub fn discount(&self, regret: Option<crate::Utility>) -> f32 {
        match regret {
            None => {
                let g = self.gamma();
                let t = self.profile().epochs() as f32;
                (t / (t + 1.)).powf(g)
            }
            Some(r) => {
                let a = self.alpha();
                let o = self.omega();
                let p = self.period() as f32;
                let t = self.profile().epochs() as f32;
                if t % p != 0. {
                    1.
                } else if r > 0. {
                    let x = (t / p).powf(a);
                    x / (x + 1.)
                } else if r < 0. {
                    let x = (t / p).powf(o);
                    x / (x + 1.)
                } else {
                    let x = t / p;
                    x / (x + 1.)
                }
            }
        }
    }

    /// Discount parameters for the training process.
    /// These values control how quickly the algorithm converges
    /// and how much weight is given to recent updates versus historical data.
    ///
    /// - `alpha`: Controls the rate at which recent updates are given more weight.
    /// - `omega`: Controls the rate at which historical updates are given more weight.
    const fn alpha(&self) -> f32 {
        1.5
    }
    const fn omega(&self) -> f32 {
        0.5
    }
    const fn gamma(&self) -> f32 {
        1.5
    }
    const fn period(&self) -> usize {
        1
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
                    "    {:?}  R {:>+6.2}, W {:>6.2}, P {:>6.2},  A {:>6.2}",
                    edge,
                    self.profile().regret(turn, edge),
                    self.profile().weight(turn, edge),
                    self.profile().policy(turn, edge),
                    self.profile().advice(turn, edge),
                )?;
            }
        }
        Ok(())
    }
}
