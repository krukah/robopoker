//! How strategies are encoded and stored: [`Encoding`] buckets game states,
//! [`Profile`] holds accumulated regrets and policies, and [`Equilibrium`] /
//! [`Flow`] blanket the read-only queries and the CFR math on top of it.
//!
//! [`Solution`] is the supertrait combining every capability.

mod access;
mod decision;
mod encoding;
mod infoset;
mod math;
mod posterior;

pub use access::*;
pub use decision::*;
pub use encoding::*;
pub use infoset::*;
pub use math::*;
pub use posterior::*;
