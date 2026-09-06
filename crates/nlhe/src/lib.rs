//! No-Limit Hold'em instantiation of the generic MCCFR traits.
//!
//! Bridges `kicker` (poker rules) and `mccfr` (generic CFR). The `Nlhe*` type
//! aliases mark exactly which kicker types cross into CFR.
//!
//! Two abstractions keep the state space tractable: suit isomorphism plus
//! k-means over equity distributions on the private side, and a street-dependent
//! grid of pot-fraction raise sizes (see `Info::raises`) on the action side.

#[cfg(feature = "server")]
mod adapt;
mod edge;
mod encoder;
mod flagship;
mod game;
mod info;
#[cfg(feature = "server")]
mod lazy;
#[cfg(feature = "server")]
mod lookup;
mod memory;
mod nest;
#[cfg(feature = "server")]
mod profile;
mod public;
mod record;
mod secret;
#[cfg(feature = "server")]
mod sink;
mod solver;
#[cfg(feature = "server")]
mod source;
mod strategy;
mod turn;
#[cfg(feature = "server")]
mod wire;

#[cfg(feature = "server")]
pub use adapt::*;
pub use edge::*;
pub use encoder::*;
pub use flagship::*;
pub use game::*;
pub use info::*;
#[cfg(feature = "server")]
pub use lazy::*;
#[cfg(feature = "server")]
pub use lookup::*;
pub use memory::*;
pub use nest::*;
pub use public::*;
pub use record::*;
pub use secret::*;
#[cfg(feature = "server")]
pub use sink::*;
pub use solver::*;
#[cfg(feature = "server")]
pub use source::*;
pub use strategy::*;
pub use turn::*;
#[cfg(feature = "server")]
pub use wire::*;

/// Flagship NLHE solver, configured to match Pluribus (Brown & Sandholm,
/// Science 2019). [`mccfr::LinearRegret`] is Linear CFR — DCFR(1, 1, 1) — the
/// variant Pluribus actually used, and is a deliberate parity choice.
pub type Flagship = Nlhe<
    mccfr::LinearRegret,     //
    mccfr::LinearWeight,     //
    mccfr::PluribusSampling, //
>;
