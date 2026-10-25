//! Sampling strategies for MCCFR tree traversal: which branches [`TreeBuilder`] explores,
//! trading off variance against convergence speed and cost.
//!
//! All are zero-cost unit structs selected via the [`Solver::S`] associated type.

mod scheme;
mod uniform;
pub use scheme::*;
pub(crate) use uniform::*;

mod external;
mod pluribus;
mod pruning;
mod targeted;
mod vanilla;

pub use external::*;
pub use pluribus::*;
pub use pruning::*;
pub use targeted::*;
pub use vanilla::*;

use crate::*;
