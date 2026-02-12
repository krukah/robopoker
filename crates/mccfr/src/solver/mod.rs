//! CFR training algorithm and utilities.
//!
//! This module defines the training loop and supporting structures:
//! - Solver: orchestrates tree sampling and regret updates
//! - TreeBuilder: lazy iterator-based tree construction
//! - Counterfactual: update vectors for regret/policy
//! - Trajectory: path replay for reach calculations

mod builder;
mod counterfactual;
mod encounter;
mod solver;
mod trajectory;
mod waypoint;
mod waypoints;

pub use builder::*;
pub use counterfactual::*;
pub use encounter::*;
pub use solver::*;
pub use trajectory::*;
pub use waypoint::*;
pub use waypoints::*;
