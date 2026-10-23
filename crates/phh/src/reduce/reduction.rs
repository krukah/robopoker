use super::*;

/// A hand reduced to our tree, with the numbers a projection needs.
#[derive(Debug, Clone)]
pub struct Reduction {
    seats: Seats,
    /// Chips in the pot when the flop was dealt.
    pot: Amount,
    /// Chips in that pot contributed by players who had already folded.
    dead: Amount,
    /// Preflop raises made by anyone at the table, capping the depth axis.
    depth: usize,
    /// The last preflop aggressor, when there was one.
    aggressor: Option<usize>,
    /// Smaller of the two stacks behind at the flop.
    effective: Amount,
}
impl Reduction {
    pub fn new(
        seats: Seats,
        pot: Amount,
        dead: Amount,
        depth: usize,
        aggressor: Option<usize>,
        effective: Amount,
    ) -> Self {
        Self {
            seats,
            pot,
            dead,
            depth,
            aggressor,
            effective,
        }
    }

    pub fn seats(&self) -> Seats {
        self.seats
    }

    pub fn pot(&self) -> Amount {
        self.pot
    }

    pub fn dead(&self) -> Amount {
        self.dead
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn aggressor(&self) -> Option<usize> {
        self.aggressor
    }

    pub fn effective(&self) -> Amount {
        self.effective
    }

    /// Flop pot in big blinds — the quantity a projected heads-up preflop path
    /// has to match.
    pub fn pot_bb(&self, bblind: Amount) -> f64 {
        f64::from(self.pot()) / f64::from(bblind.max(1))
    }

    /// Effective stack in big blinds.
    pub fn effective_bb(&self, bblind: Amount) -> f64 {
        f64::from(self.effective()) / f64::from(bblind.max(1))
    }

    /// True when the in-position player took the last preflop aggression.
    pub fn ip_is_aggressor(&self) -> bool {
        self.aggressor() == Some(self.seats().ip())
    }

    /// Hand-built reduction, for tests that need one without a whole hand.
    #[cfg(test)]
    pub(crate) fn sample(oop: usize, ip: usize, pot: Amount, depth: usize, aggressor: Option<usize>) -> Self {
        Self {
            seats: Seats::new(oop, ip),
            pot,
            dead: 0,
            depth,
            aggressor,
            effective: 10_000 - pot / 2,
        }
    }
}

impl Reduction {}
