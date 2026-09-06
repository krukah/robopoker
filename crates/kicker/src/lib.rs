//! No-Limit Texas Hold'em rules and mechanics: state across betting rounds,
//! action handling, and showdown settlement.
//!
//! [`Game`] is the memoryless present. [`Witness`] and [`Perfect`] are the
//! remembered past at two information levels — hero's cards only versus both
//! players' — and [`Path`] is their compressed edge sequence. Concrete
//! [`Action`]s abstract into [`Edge`]s for strategy lookup.
mod abstraction;
mod action;
mod arrangement;
mod axis;
mod bias;
pub mod dto;
mod edge;
mod field;
mod game;
mod geometry;
mod grid;
mod live;
mod mask;
mod message;
mod odds;
mod path;
mod perfect;
mod pnl;
mod raise;
mod recall;
mod seat;
mod settlement;
mod showdown;
mod size;
mod snapshot;
mod subgame;
mod turn;
mod witness;

pub use abstraction::*;
pub use action::*;
pub use arrangement::*;
pub use axis::*;
pub use bias::*;
pub use dto::*;
pub use edge::*;
pub use field::*;
pub use game::*;
pub use geometry::*;
pub use grid::*;
pub use live::*;
pub use mask::*;
pub use message::*;
pub use odds::*;
pub use path::*;
pub use perfect::*;
pub use pnl::*;
pub use raise::*;
pub use recall::*;
pub use seat::*;
pub use settlement::*;
pub use showdown::*;
pub use size::*;
pub use snapshot::*;
pub use subgame::*;
pub use turn::*;
pub use witness::*;
