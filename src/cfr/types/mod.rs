//! Types module containing simple type aliases and wrappers
//!
//! This module contains shallow types that have no associated logic and simply
//! alias tuples or standard collections. These types are used throughout the CFR
//! implementation to provide more semantic naming and improve code readability,
//! while maintaining the simplicity of basic Rust types underneath.

pub mod branch;
pub mod counterfactual;
pub mod policy;
