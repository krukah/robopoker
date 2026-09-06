//! How strategies are encoded and stored: [`CfrEncoder`] buckets game states,
//! [`RefProf`] holds accumulated regrets and policies, and [`CfrNash`] /
//! [`CfrFlow`] blanket the read-only queries and the CFR math on top of it.
//!
//! [`CfrSolution`] is the supertrait combining every capability.

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
