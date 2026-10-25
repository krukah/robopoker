use super::*;
use deuce::*;

/// A decision node, capturing the state *before* the logged action.
///
/// Everything Tier 1 (abstraction calibration) and Tier 2 (policy agreement)
/// need about a betting decision, in the engine's own accounting terms.
#[derive(Debug, Clone)]
pub struct Node {
    seat: usize,
    street: Street,
    depth: usize,
    pot: Amount,
    stake: Amount,
    top: Amount,
    stack: Amount,
    alive: usize,
    step: Step,
}
impl Node {
    pub fn new(
        seat: usize,
        street: Street,
        depth: usize,
        pot: Amount,
        stake: Amount,
        top: Amount,
        stack: Amount,
        alive: usize,
        step: Step,
    ) -> Self {
        Self {
            seat,
            street,
            depth,
            pot,
            stake,
            top,
            stack,
            alive,
            step,
        }
    }

    pub fn seat(&self) -> usize {
        self.seat
    }

    pub fn street(&self) -> Street {
        self.street
    }

    /// Raises already made on this street — the `depth` axis of the bet grid.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Total chips in the middle before this action, current street included.
    /// Matches `kicker::GameN::pot`.
    pub fn pot(&self) -> Amount {
        self.pot
    }

    /// The actor's wager already on this street.
    pub fn stake(&self) -> Amount {
        self.stake
    }

    /// The largest wager on this street.
    pub fn top(&self) -> Amount {
        self.top
    }

    /// The actor's chips behind, before acting.
    pub fn stack(&self) -> Amount {
        self.stack
    }

    /// Players not yet folded, this actor included.
    pub fn alive(&self) -> usize {
        self.alive
    }

    /// What the player actually did.
    pub fn step(&self) -> &Step {
        &self.step
    }

    pub fn to_call(&self) -> Amount {
        self.top() - self.stake()
    }

    /// Chips the actor puts in now. This is the quantity `kicker` denominates
    /// bet sizes against: `Action::Raise(chips)` is an increment from the
    /// actor's current stake, not a to-amount.
    pub fn put(&self) -> Amount {
        match self.step() {
            Step::BetTo(_, to) => to - self.stake(),
            Step::CheckCall(_) => self.to_call(),
            _ => 0,
        }
    }

    /// True when the action commits the actor's last chip.
    pub fn is_shove(&self) -> bool {
        self.put() >= self.stack()
    }

    /// Raise size on the engine's pot-fraction axis: chips in now over the pot
    /// before the action. Mirrors `Size::translate`'s `Grid::Postflop` branch.
    pub fn pot_fraction(&self) -> f64 {
        f64::from(self.put()) / f64::from(self.pot().max(1))
    }

    /// Raise size on the engine's BB axis, used for preflop opens. Mirrors
    /// `Size::translate`'s `Grid::Opening` branch, where `Edge::Open(n)` puts
    /// in `n` big blinds.
    pub fn bb_units(&self, bblind: Amount) -> f64 {
        f64::from(self.put()) / f64::from(bblind.max(1))
    }
}

impl Node {}
