//! Game-agnostic CFR abstractions and machinery.
//!
//! This module contains all the generic traits and types that define the
//! CFR algorithm independently of any specific game implementation.
//!
//! # Module Structure
//!
//! - `state` — State primitives (Turn, Edge, Game, Info, Tree)
//! - `strategy` — Strategy representation (Encoder, Profile, InfoSet)
//! - `solver` — Training algorithm (Solver, TreeBuilder, Counterfactual)
//! - `policy` — Strategy weighting schemes
//! - `regret` — Regret update schemes
//! - `sample` — Sampling schemes
//! - `subgame` — Safe subgame solving
//! - `metrics` — Training observability
//! - `rps` — Rock-Paper-Scissors reference implementation

mod metrics;
mod policy;
mod regret;
mod rps;
mod sample;
mod solver;
mod state;
mod strategy;
mod subgame;

pub use metrics::*;
pub use policy::*;
pub use regret::*;
pub use rps::*;
pub use sample::*;
pub use solver::*;
pub use state::*;
pub use strategy::*;
pub use subgame::*;
