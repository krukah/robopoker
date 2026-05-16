//! Distributed training workers for MCCFR.
mod pool;
mod worker;
pub use pool::*;
pub use rbp_nlhe::Memory;
pub use rbp_nlhe::Record;
pub use rbp_mccfr::Progress;
pub use worker::*;
