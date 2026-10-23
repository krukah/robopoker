//! One geometry, two orientations, many frames — generic up to the mccfr trait hierarchy.
//!
//! The reel renders a solver game's tree as a scripted animation: an authored hand explored
//! phase by phase, one trajectory per frame, folds and shoves probing the subgame before the
//! chosen line freezes into the white prefix.
mod canvas;
mod color;
mod game;
mod geometry;
mod motion;
mod panels;
mod script;

pub use canvas::*;
pub use color::*;
pub use game::*;
pub use geometry::*;
pub use motion::*;
pub use panels::*;
pub use script::*;
