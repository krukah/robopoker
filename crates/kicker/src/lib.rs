//! No-Limit Texas Hold'em rules and mechanics: state across betting rounds, action handling,
//! and showdown settlement.
//!
//! [`Game`] is the memoryless present. [`Witness`] and [`Perfect`] are the remembered past at
//! two information levels — hero's cards only versus both players' — and [`Path`] is their
//! compressed edge sequence. Concrete [`Action`]s abstract into [`Edge`]s for strategy lookup.
mod betting;
pub mod dto;
mod history;
mod info;
mod rollout;
mod room;
mod settle;
mod table;
pub use betting::*;
pub use dto::*;
pub use history::*;
pub use info::*;
pub use rollout::*;
pub use room::*;
pub use settle::*;
pub use table::*;
