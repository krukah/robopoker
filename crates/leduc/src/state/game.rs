/// Position within a betting round.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Spot {
    Open,
    Checked,
    Raised,
    CheckRaised,
}

impl Spot {
    pub fn raised(&self) -> bool {
        matches!(self, Spot::Raised | Spot::CheckRaised)
    }

    pub(crate) fn actor(&self) -> usize {
        match self {
            Spot::Open | Spot::CheckRaised => 0,
            Spot::Checked | Spot::Raised => 1,
        }
    }
}
