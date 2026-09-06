//! The CFR training loop: [`Solver`] orchestrates tree sampling and regret
//! updates over [`TreeBuilder`] trees and [`Decisions`] update vectors.

mod builder;
mod decisions;
mod encounter;
mod harvest;
#[allow(clippy::module_inception)]
mod solver;

pub use builder::*;
pub use decisions::*;
pub use encounter::*;
pub use harvest::*;
pub use solver::*;
