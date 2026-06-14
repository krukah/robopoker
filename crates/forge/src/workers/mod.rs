//! Distributed training workers for MCCFR.
mod pool;
mod worker;
pub use holdem::Memory;
pub use holdem::Record;
pub use pool::*;
pub use regret::Progress;
pub use worker::*;
