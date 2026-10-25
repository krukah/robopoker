#![doc = include_str!("doc/players.md")]

#[cfg(feature = "server")]
mod bots;
mod fish;
#[cfg(feature = "cli")]
mod human;
mod players;
mod roster;

#[cfg(feature = "server")]
pub use bots::*;
pub use fish::*;
#[cfg(feature = "cli")]
pub use human::*;
pub use players::*;
pub use roster::*;
