use pokerkit::*;

/// Chips at a node: the pot, what the actor owes to call, every seat's stack behind, and the
/// effective stack among players still in.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ledger {
    pot: Chips,
    call: Chips,
    stacks: [Chips; N],
    effective: Chips,
}

impl Ledger {
    pub fn new(pot: Chips, call: Chips, stacks: [Chips; N], effective: Chips) -> Self {
        Self {
            pot,
            call,
            stacks,
            effective,
        }
    }

    pub fn pot(&self) -> Chips {
        self.pot
    }

    pub fn call(&self) -> Chips {
        self.call
    }

    pub fn stacks(&self) -> [Chips; N] {
        self.stacks
    }

    pub fn effective(&self) -> Chips {
        self.effective
    }
}
