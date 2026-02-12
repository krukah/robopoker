//! Strategy representation and abstraction.
//!
//! This module defines how strategies are encoded and stored:
//! - Encoder: maps game states to information set buckets
//! - Profile: stores accumulated regrets and policies
//! - InfoSet: groups tree nodes sharing the same information
//! - Decision: action-weight pairs for strategy construction
//! - Posterior: Bayesian belief over opponent's private information

mod decision;
mod encoder;
mod infoset;
mod posterior;
mod profile;

pub use decision::*;
pub use encoder::*;
pub use infoset::*;
pub use posterior::*;
pub use profile::*;
