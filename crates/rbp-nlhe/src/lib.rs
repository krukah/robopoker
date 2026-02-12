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
//! - [`Info`] — Information set: public + private state
//! - [`NlheEncoder`] — Maps game states to [`Info`] using clustering
//! - [`NlheProfile`] — Stores accumulated regrets and strategies
//! - [`NlheSolver`] — Generic solver combining encoder and profile
//! - [`Flagship`](crate::Flagship) — Pluribus-configured solver (top-level alias)
//!
//! # Abstraction
//!
//! The key challenge in poker CFR is the enormous state space. This module
//! uses strategic abstraction via the [`Isomorphism`] to [`Abstraction`] mapping:
//! - Suit-isomorphic hands collapse equivalent situations
//! - K-means clustering groups similar equity distributions
//!
//! # Action Space
//!
//! Betting amounts are discretized into a street-dependent grid of pot-fraction
//! raise sizes (see [`Info::raises`]). This keeps the action space tractable
//! while preserving strategically important bet sizes.

mod edge;
mod encoder;
mod game;
mod info;
mod profile;
mod public;
mod secret;
mod solver;
mod strategy;
mod turn;

pub use edge::*;
pub use encoder::*;
pub use game::*;
pub use info::*;
pub use profile::*;
pub use public::*;
pub use secret::*;
pub use solver::*;
pub use strategy::*;
pub use turn::*;

/// Flagship NLHE solver configuration.
///
/// Uses the Pluribus algorithm configuration:
/// - [`rbp_mccfr::PluribusSampling`] — Probabilistic pruning with warm-up period
/// - [`rbp_mccfr::PluribusRegret`] — No discount for positive regrets, t/(t+1) for negative
/// - [`rbp_mccfr::LinearWeight`] — Emphasize more recent iterations in average strategy
pub type Flagship = NlheSolver<
    rbp_mccfr::PluribusRegret,   //
    rbp_mccfr::LinearWeight,     //
    rbp_mccfr::PluribusSampling, //
>;
