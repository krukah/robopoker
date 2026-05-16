//! CFR training algorithm and utilities.
//!
//! This module defines the training loop and supporting structures:
//! - Solver: orchestrates tree sampling and regret updates
//! - TreeBuilder: lazy iterator-based tree construction
//! - Decisions: update vectors for regret/policy

mod builder;
mod decisions;
mod encounter;
mod harvest;
mod solver;

pub use builder::*;
pub use decisions::*;
pub use encounter::*;
pub use harvest::*;
pub use solver::*;
