//! No-Limit Hold'em specific CFR implementation.
//!
//! This module provides the concrete types needed to apply MCCFR to
//! No-Limit Texas Hold'em poker. It implements the abstract CFR traits
//! with poker-specific game rules, information abstraction, and betting.
//!
//! # Architecture
//!
//! This module serves as a bridge between `gameplay` (core poker) and `mccfr`
//! (generic CFR). Type aliases (`NlheEdge`, `NlheTurn`, etc.) make explicit
//! which gameplay types are being used for CFR, preparing for potential
//! crate separation into `nlhe`, `mccfr`, and `nlhe-mccfr`.
//!
//! # Components
//!
//! - [`NlheEdge`] — Discretized betting action (type alias for `Edge`)
//! - [`NlheTurn`] — Player indicator (type alias for `Turn`)
//! - [`NlheGame`] — Game state (type alias for `Game`)
//! - [`NlheSecret`] — Private state (type alias for `Abstraction`)
//! - [`NlhePublic`] — Public state: street + current-street edges
//! - `Info` — Information set: public + private state
//! - [`NlheEncoder`] — Maps game states to `Info` using clustering
//! - [`NlheProfile`] — Stores accumulated regrets and strategies
//! - [`Nlhe`] — Generic solver combining encoder and profile
//! - [`Flagship`] — Pluribus-configured solver (top-level alias)
//!
//! # Abstraction
//!
//! The key challenge in poker CFR is the enormous state space. This module
//! uses strategic abstraction via the `Isomorphism` to `Abstraction` mapping:
//! - Suit-isomorphic hands collapse equivalent situations
//! - K-means clustering groups similar equity distributions
//!
//! # Action Space
//!
//! Betting amounts are discretized into a street-dependent grid of pot-fraction
//! raise sizes (see `Info::raises`). This keeps the action space tractable
//! while preserving strategically important bet sizes.

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

/// Flagship NLHE solver configuration.
///
/// Matches the Pluribus (Brown & Sandholm, Science 2019) algorithm configuration:
/// - [`mccfr::LinearRegret`] — Linear CFR, i.e. DCFR(1, 1, 1), the variant Pluribus actually used
/// - [`mccfr::LinearWeight`] — Linear weighting of the average strategy
/// - [`mccfr::PluribusSampling`] — Probabilistic pruning with warm-up period
pub type Flagship = Nlhe<
    mccfr::LinearRegret,     //
    mccfr::LinearWeight,     //
    mccfr::PluribusSampling, //
>;
