use super::*;
use crate::*;
use pokerkit::Utility;

/// A counter game: state is depth, every edge advances by one.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct G(pub u8);

impl CfrGame for G {
    type E = E;
    type T = T;

    fn root() -> Self {
        G(0)
    }

    fn turn(&self) -> T {
        match self.0 {
            2 => T::Chance,
            d if d >= 4 => T::Terminal,
            _ => T::Choice,
        }
    }

    fn apply(&self, _: E) -> Self {
        G(self.0 + 1)
    }

    fn payoff(&self, _: T) -> Utility {
        Utility::from(self.0)
    }
}
