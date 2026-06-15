//! Strategy representation and abstraction.
//!
//! This module defines how strategies are encoded and stored:
//! - CfrEncoder: maps game states to information set buckets
//! - Profile: stores accumulated regrets and policies
//! - CfrNash: read-only Nash strategy queries (blanket from Profile)
//! - CfrSampling: walker identity and sampling parameters
//! - CfrFlow: regret/value flow (blanket from Profile + CfrSampling)
//! - CfrSolution: convenience supertrait combining all capabilities
//! - AsyncProfile: async variant for database-backed training
//! - InfoSet: groups tree nodes sharing the same information
//! - Decision: action-weight pairs for strategy construction
//! - Posterior: Bayesian belief over opponent's private information

#[cfg(feature = "async")]
mod async_profile;
mod book;
mod decision;
mod encoder;
mod flow;
mod infoset;
mod macros;
mod nash;
mod posterior;
mod profile;
mod property;
mod solution;
mod storage;
mod training;

#[cfg(feature = "async")]
pub use async_profile::*;
pub use book::*;
pub use decision::*;
pub use encoder::*;
pub use flow::*;
pub use infoset::*;
pub use nash::*;
pub use posterior::*;
pub use profile::*;
pub use property::*;
pub use solution::*;
pub use storage::*;
pub use training::*;
