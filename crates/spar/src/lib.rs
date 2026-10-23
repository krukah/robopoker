//! Slumbot API benchmark client.
//!
//! The [`Rival`](bout::Rival) implementation for `slumbot.com` — the research bot that has been
//! the field's reference opponent since 2016. Everything that isn't Slumbot-specific (the hand
//! loop, statistics, hand-history rows, the scoreboard) lives in [`bout`]; this crate is the
//! transport, the action string grammar, and the chip-scale translation.

mod act;
mod new_hand;
mod response;
pub(crate) use act::*;
pub(crate) use new_hand::*;
pub use response::*;
pub mod client;
pub mod hand;
pub mod session;
pub mod translate;

pub use client::*;
pub use hand::*;
pub use session::*;
pub use translate::*;
