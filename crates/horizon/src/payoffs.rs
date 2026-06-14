//! L×L payoff matrix for the frontier normal-form game.
use crate::Continuation;
use fulcrum::Utility;

/// L×L payoff matrix indexed by continuation strategy pairs.
#[derive(Debug, Clone, Copy)]
pub struct Payoffs<const D: usize>([[Utility; D]; D]);

impl<const D: usize> Payoffs<D> {
    pub fn uniform(value: Utility) -> Self {
        Self([[value; D]; D])
    }

    pub fn tabulate(f: impl Fn(Continuation, Continuation) -> Utility) -> Self {
        use std::array::from_fn;
        let cs = Continuation::all::<D>().collect::<Vec<_>>();
        Self(from_fn(|i| from_fn(|j| f(cs[i], cs[j]))))
    }

    pub fn get(&self, row: Continuation, col: Continuation) -> Utility {
        self.0[row.index()][col.index()]
    }
}
