//! The CFR training loop: [`Solver`] orchestrates tree sampling and regret
//! updates over [`TreeBuilder`] trees and [`Decisions`] update vectors.

mod harvested;
pub use harvested::*;
mod batch;
mod builder;
mod decisions;
mod encounter;
mod grower;
mod harvest;
mod shard;
#[allow(clippy::module_inception)]
mod solver;
mod tabular;

pub use batch::*;
pub use builder::*;
pub use decisions::*;
pub use encounter::*;
pub use grower::*;
pub use harvest::*;
pub use shard::*;
pub use solver::*;
pub use tabular::*;
