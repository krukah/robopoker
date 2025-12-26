use super::*;
use crate::mccfr::*;
use crate::*;
use std::collections::BTreeMap;

#[derive(Default)]
/// Blueprint represents the complete solution for the Rock Paper Scissors game,
/// storing accumulated regret and policy values that are built up during training.
/// It implements both Profile (for tracking historical regrets and policies) and
/// Trainer (for using those values to optimize strategy through counterfactual regret minimization).
/// As a Profile, it provides access to the current state. As a Trainer, it updates that state
/// to converge toward Nash equilibrium.
pub struct RpsSolver {
    epochs: usize,
    encounters: BTreeMap<RpsTurn, BTreeMap<RpsEdge, (Probability, Utility)>>,
}

/// For the Rock Paper Scissors game, encoding is straightforward
/// since there are only three possible moves.
impl Encoder for RpsSolver {
    type T = RpsTurn;
    type E = RpsEdge;
    type G = RpsGame;
    type I = RpsTurn;

    fn seed(&self, _: &Self::G) -> Self::I {
        RpsTurn::P1
    }

    fn info(
        &self,
        _: &Tree<Self::T, Self::E, Self::G, Self::I>,
        (_, game, _): Branch<Self::E, Self::G>,
    ) -> Self::I {
        game.turn()
    }
}

/// For the Rock Paper Scissors game, Blueprint implements the Profile trait.
/// It tracks regrets and policies over time.
impl Profile for RpsSolver {
    type T = RpsTurn;
    type E = RpsEdge;
    type G = RpsGame;
    type I = RpsTurn;

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

    fn sum_policy(&self, info: &Self::I, edge: &Self::E) -> crate::Probability {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(w, _)| *w)
            .unwrap_or_default()
    }

    fn sum_regret(&self, info: &Self::I, edge: &Self::E) -> crate::Utility {
        self.encounters
            .get(info)
            .and_then(|memory| memory.get(edge))
            .map(|(_, r)| *r)
            .unwrap_or_default()
    }
}

/// For the Rock Paper Scissors game, Blueprint implements both Trainer and Profile traits.
/// As a Profile, it tracks regrets and policies over time. As a Trainer, it uses those
/// values to train an optimal strategy through counterfactual regret minimization.
impl Blueprint for RpsSolver {
    type T = RpsTurn;
    type E = RpsEdge;
    type G = RpsGame;
    type I = RpsTurn;
    type P = Self;
    type S = Self;

    fn tree_count() -> usize {
        CFR_TREE_COUNT_RPS
    }
    fn batch_size() -> usize {
        CFR_BATCH_SIZE_RPS
    }

    fn encoder(&self) -> &Self::S {
        &self
    }

    fn profile(&self) -> &Self::P {
        &self
    }

    fn mut_policy(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(info.clone())
            .or_insert_with(BTreeMap::default)
            .entry(edge.clone())
            .or_insert((0., 0.))
            .0
    }

    fn mut_regret(&mut self, info: &Self::I, edge: &Self::E) -> &mut f32 {
        &mut self
            .encounters
            .entry(info.clone())
            .or_insert_with(BTreeMap::default)
            .entry(edge.clone())
            .or_insert((0., 0.))
            .1
    }

    fn advance(&mut self) {
        Profile::increment(self)
    }
}

#[rustfmt::skip]
impl std::fmt::Display for RpsSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Epochs: {}", self.epochs)?;
        writeln!(f, "┌──────┬──────┬──────────┬──────────┬──────────┬──────────┐")?;
        writeln!(f, "│ Turn │ Edge │   Regret │   Weight │   Policy │   Advice │")?;
        writeln!(f, "├──────┼──────┼──────────┼──────────┼──────────┼──────────┤")?;
        for (turn, edges) in &self.encounters {
            for (edge, _) in edges {
                writeln!(
                    f,
                    "│ {:>4} │ {:>4} │ {:>+8.2} │ {:>8.2} │ {:>8.2} │ {:>8.2} │",
                    format!("{:?}", turn),
                    format!("{:?}", edge),
                    self.profile().sum_regret(turn, edge),
                    self.profile().sum_policy(turn, edge),
                    self.profile().matching(turn, edge),
                    self.profile().averaged(turn, edge),
                )?;
            }
        }
        writeln!(f, "└──────┴──────┴──────────┴──────────┴──────────┴──────────┘")?;
        Ok(())
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mccfr::cache::CachedBlueprint;
    pub const TOLERANCE: f32 = 0.05;

    #[test]
    fn converge() {
        let solution = RpsSolver::default().solve();
        println!("{}", solution);
        let profile = solution.profile();
        for turn in [RpsTurn::P1, RpsTurn::P2] {
            let r = profile.averaged(&turn, &RpsEdge::R);
            let p = profile.averaged(&turn, &RpsEdge::P);
            let s = profile.averaged(&turn, &RpsEdge::S);
            assert!((r - 0.40).abs() < TOLERANCE, "{:?} R: {} not near 0.40", turn, r);
            assert!((p - 0.40).abs() < TOLERANCE, "{:?} P: {} not near 0.40", turn, p);
            assert!((s - 0.20).abs() < TOLERANCE, "{:?} S: {} not near 0.20", turn, s);
        }
    }

    #[test]
    fn cached_converge() {
        let solution = CachedBlueprint::new(RpsSolver::default()).solve().into_inner();
        println!("{}", solution);
        let profile = solution.profile();
        for turn in [RpsTurn::P1, RpsTurn::P2] {
            let r = profile.averaged(&turn, &RpsEdge::R);
            let p = profile.averaged(&turn, &RpsEdge::P);
            let s = profile.averaged(&turn, &RpsEdge::S);
            assert!((r - 0.40).abs() < TOLERANCE, "{:?} R: {} not near 0.40", turn, r);
            assert!((p - 0.40).abs() < TOLERANCE, "{:?} P: {} not near 0.40", turn, p);
            assert!((s - 0.20).abs() < TOLERANCE, "{:?} S: {} not near 0.20", turn, s);
        }
    }

    /// Both implementations converge to the same equilibrium but use different
    /// random samples, so we allow tolerance matching the convergence test.
    #[test]
    fn cached_matches_uncached() {
        let uncached = RpsSolver::default().solve();
        let cached = CachedBlueprint::new(RpsSolver::default()).solve().into_inner();
        for turn in [RpsTurn::P1, RpsTurn::P2] {
            for edge in [RpsEdge::R, RpsEdge::P, RpsEdge::S] {
                let uncached_advice = uncached.profile().averaged(&turn, &edge);
                let cached_advice = cached.profile().averaged(&turn, &edge);
                let diff = (uncached_advice - cached_advice).abs();
                assert!(diff < TOLERANCE,
                    "{:?}/{:?}: uncached={:.4} cached={:.4} diff={:.4}",
                    turn, edge, uncached_advice, cached_advice, diff);
            }
        }
    }
}
