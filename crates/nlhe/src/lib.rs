//! No-Limit Hold'em instantiation of the generic MCCFR traits.
//!
//! Bridges `kicker` (poker rules) and `mccfr` (generic CFR). The `Nlhe*` type aliases mark
//! exactly which kicker types cross into CFR.

mod blueprint;
mod cfr;
mod state;
#[cfg(feature = "server")]
mod store;

pub use blueprint::*;
pub use cfr::*;
pub use state::*;
#[cfg(feature = "server")]
pub use store::*;
