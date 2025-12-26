//! Types module containing simple type aliases and wrappers
//!
//! This module contains shallow types that have no associated logic and simply
//! alias tuples or standard collections. These types are used throughout the CFR
//! implementation to provide more semantic naming and improve code readability,
//! while maintaining the simplicity of basic Rust types underneath.

mod branch;
mod counterfactual;
mod decision;
mod policy;
mod strategy;

pub use branch::*;
pub use counterfactual::*;
pub use decision::*;
pub use policy::*;
pub use strategy::*;
