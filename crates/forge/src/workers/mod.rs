//! Distributed training workers for MCCFR.
mod pool;
mod worker;
pub use mccfr::Progress;
pub use nlhe::Memory;
pub use nlhe::Record;
pub use pool::*;
pub use worker::*;
