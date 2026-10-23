use pokerkit::*;

/// What a line was worth at one spot, averaged over a shared set of worlds.
///
/// A single action's wager weights every world equally. A whole strategy's
/// weights each of its actions by the mass it puts there, which is why the
/// average is carried as a weighted sum rather than a count.
#[derive(Debug, Clone, Copy, Default)]
pub struct Wager {
    total: f64,
    weight: f64,
    worlds: usize,
    misses: usize,
}
impl Wager {
    pub fn new(total: f64, weight: f64, worlds: usize, misses: usize) -> Self {
        Self {
            total,
            weight,
            worlds,
            misses,
        }
    }

    /// How many rollouts went into the mean.
    pub fn worlds_mut(&mut self) -> &mut usize {
        &mut self.worlds
    }

    pub fn worlds(&self) -> usize {
        self.worlds
    }

    /// Decisions inside the rollout where the strategy had no distribution and
    /// the check-or-call fallback answered for it.
    pub fn misses_mut(&mut self) -> &mut usize {
        &mut self.misses
    }

    pub fn misses(&self) -> usize {
        self.misses
    }

    /// Mean big blinds won over the whole hand.
    ///
    /// Absolute, so it carries the chips committed before the decision — which
    /// are identical in both branches of a [`crate::Duel`] and cancel out of
    /// [`crate::Duel::delta`]. Only the difference is interpretable.
    pub fn value(&self) -> Utility {
        match self.weight() {
            weight if weight > 0.0 => (self.total() / weight) as Utility,
            _ => 0.0,
        }
    }
}
impl std::ops::Add for Wager {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            total: self.total() + other.total(),
            weight: self.weight() + other.weight(),
            worlds: self.worlds() + other.worlds(),
            misses: self.misses() + other.misses(),
        }
    }
}
impl std::iter::Sum for Wager {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::default(), std::ops::Add::add)
    }
}

impl Wager {
    pub fn total_mut(&mut self) -> &mut f64 {
        &mut self.total
    }

    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn weight_mut(&mut self) -> &mut f64 {
        &mut self.weight
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }
}
