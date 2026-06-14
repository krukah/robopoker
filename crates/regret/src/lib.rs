//! Game-agnostic CFR abstractions and machinery.
//!
//! This module contains all the generic traits and types that define the
//! CFR algorithm independently of any specific game implementation.
//!
//! # Module Structure
//!
//! - `state` — State primitives (Turn, Edge, Game, Info, Tree)
//! - `strategy` — Strategy representation (CfrEncoder, Profile, InfoSet)
//! - `solver` — Training algorithm (Solver, TreeBuilder, Decisions)
//! - `policy` — Strategy weighting schemes
//! - `regret` — Regret update schemes
//! - `sample` — Sampling schemes
//! - `metrics` — Training observability

mod hyperparams;
mod metrics;
mod policy;
mod regret;
mod sample;
mod solver;
mod state;
mod strategy;

pub use hyperparams::*;
pub use metrics::*;
pub use policy::*;
pub use regret::*;
pub use sample::*;
pub use solver::*;
pub use state::*;
pub use strategy::*;
