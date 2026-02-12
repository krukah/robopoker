//! Hand history recording and replay.
//!
//! This module captures completed poker hands for analysis, replay, and
//! persistence. Each record preserves the full action sequence and
//! participant information.
//!
//! ## Core Types
//!
//! - [`Hand`] — Complete record of a single hand
//! - [`Play`] — A single action within a hand
//! - [`Participant`] — Player identity and starting stack
//! - [`Replay`] — Iterator for stepping through recorded hands
//! - [`Room`] — Marker type for room identity
mod hand;
mod participant;
mod play;
mod replay;
mod room;

pub use hand::*;
pub use participant::*;
pub use play::*;
pub use replay::*;
pub use room::*;
