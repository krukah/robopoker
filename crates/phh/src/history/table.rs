use super::*;

/// Mutable view of the chip accounting, so the wager rule lives in one place.
pub(crate) struct Table<'chips> {
    stacks: &'chips mut Vec<Amount>,
    committed: &'chips mut Vec<Amount>,
    stakes: &'chips mut Vec<Amount>,
    pot: &'chips mut Amount,
}
impl Table<'_> {
    /// Moves `amount` (capped at the seat's stack) from stack to pot.
    pub(crate) fn wager(&mut self, seat: usize, amount: Amount) {
        let amount = amount.clamp(0, self.stacks()[seat]);
        self.stacks_mut()[seat] -= amount;
        self.committed_mut()[seat] += amount;
        self.stakes_mut()[seat] += amount;
        *self.pot_mut() += amount;
    }
}

impl Table<'_> {
    pub fn stacks(&self) -> &[Amount] {
        self.stacks
    }

    fn stacks_mut(&mut self) -> &mut Vec<Amount> {
        self.stacks
    }

    fn committed_mut(&mut self) -> &mut Vec<Amount> {
        self.committed
    }

    pub fn stakes(&self) -> &[Amount] {
        self.stakes
    }

    pub fn stakes_mut(&mut self) -> &mut Vec<Amount> {
        self.stakes
    }

    pub fn pot(&self) -> Amount {
        *self.pot
    }

    fn pot_mut(&mut self) -> &mut Amount {
        self.pot
    }
}

impl<'chips> Table<'chips> {
    pub fn new(
        stacks: &'chips mut Vec<Amount>,
        committed: &'chips mut Vec<Amount>,
        stakes: &'chips mut Vec<Amount>,
        pot: &'chips mut Amount,
    ) -> Self {
        Self {
            stacks,
            committed,
            stakes,
            pot,
        }
    }
}
