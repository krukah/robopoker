use crate::*;
use mccfr::*;

/// Deep CFR's three memories: one `M_V,p` of regret rows per player, and `M_Π` of strategy rows.
/// Every network keeps one, so what a net remembers never depends on how it fits.
pub struct Memories<F>
where
    F: CfrInfo,
{
    advantages: FxHashMap<TurnOf<F>, Reservoir<Sample<F, Regret>>>,
    strategies: Reservoir<Sample<F, Policy>>,
    capacity: usize,
}

impl<F> Memories<F>
where
    F: CfrInfo,
{
    /// `capacity` bounds each memory.
    pub fn new(capacity: usize) -> Self {
        Self {
            advantages: FxHashMap::default(),
            strategies: Reservoir::new(capacity),
            capacity,
        }
    }

    pub fn see_regret(&mut self, walker: TurnOf<F>, sample: Sample<F, Regret>) {
        let capacity = self.capacity;
        self.advantages
            .entry(walker)
            .or_insert_with(|| Reservoir::new(capacity))
            .insert(sample);
    }

    pub fn see_strategy(&mut self, sample: Sample<F, Policy>) {
        self.strategies.insert(sample);
    }

    /// `M_V,p`, once player `p` has walked.
    pub fn advantages(&self, walker: &TurnOf<F>) -> Option<&Reservoir<Sample<F, Regret>>> {
        self.advantages.get(walker)
    }

    pub fn strategies(&self) -> &Reservoir<Sample<F, Policy>> {
        &self.strategies
    }
}
