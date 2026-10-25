use mccfr::*;
use pokerkit::*;

/// One reservoir row of Deep CFR: an infoset, the 1-based iteration `t` that drew it, and a
/// per-edge target — sampled regret `r̃ᵗ(I,·)` for `M_V`, or the strategy `σᵗ(I)` for `M_Π`.
#[derive(Debug, Clone)]
pub struct Sample<F, K>
where
    F: CfrInfo,
    K: Meaning,
{
    t: usize,
    info: F,
    target: Over<EdgeOf<F>, K>,
}

impl<F, K> Sample<F, K>
where
    F: CfrInfo,
    K: Meaning,
{
    pub fn new(info: F, t: usize, target: Over<EdgeOf<F>, K>) -> Self {
        Self { t, info, target }
    }

    pub fn info(&self) -> &F {
        &self.info
    }

    pub fn t(&self) -> usize {
        self.t
    }

    pub fn target(&self) -> &Over<EdgeOf<F>, K> {
        &self.target
    }

    /// Linear CFR weight on this row's squared error in both Deep CFR losses.
    pub fn weight(&self) -> Utility {
        self.t() as Utility
    }
}
