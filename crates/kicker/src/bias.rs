use pokerkit::*;

/// Initial regret seed weights (warmstart bias).
///
/// Weights, not probabilities — only ratios matter. Read by
/// [`crate::Edge::regret`] when seeding CFR warmstart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiasHyperParams {
    folds: Utility,
    raise: Utility,
    shove: Utility,
    other: Utility,
}

impl BiasHyperParams {
    pub fn new(folds: Utility, raise: Utility, shove: Utility, other: Utility) -> Self {
        Self {
            folds,
            raise,
            shove,
            other,
        }
    }

    /// Initial regret weight for fold actions.
    pub fn folds(&self) -> Utility {
        self.folds
    }

    /// Initial regret weight for sized raise actions (per-action).
    pub fn raise(&self) -> Utility {
        self.raise
    }

    /// Initial regret weight for the all-in (Shove) edge.
    /// Separate from `raise()` so warmstart can bias against jamming
    /// without affecting normal raise frequencies.
    pub fn shove(&self) -> Utility {
        self.shove
    }

    /// Initial regret weight for call/check actions.
    pub fn other(&self) -> Utility {
        self.other
    }
}

impl Default for BiasHyperParams {
    fn default() -> Self {
        // Scaled two orders of magnitude above per-visit regret swings so the
        // warmstart persists for ~hundreds of visits instead of being washed
        // out on the first update. Fold mass is bumped harder because folding
        // is underrated (and raising overrated) under the smaller seed.
        // Per-action raise weight × ~5 aggressive edges ≈ 100 ≈ other mass,
        // preserving ~50/50 bet/check intent; fold mass dominates
        // fold/call/raise spots at ~50/25/25.
        //
        // Shove starts at zero — no positive warmstart bias. CFR has to
        // earn jamming via accumulated regret rather than be steered toward
        // it during exploration. This addresses the empirical pattern where
        // the action grid's max non-jam size is far enough below all-in that
        // CFR over-favored ! during convergence.
        Self {
            folds: 100.0,
            raise: 10.0,
            shove: 0.0,
            other: 50.0,
        }
    }
}

pokerkit::hyperparams!(BiasHyperParams);
