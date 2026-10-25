//! Combined safe subgame solving + depth-limited frontiers.
//!
//! A thin composition layer: [`SubGameEncoder`] tags info sets with a world *and* detects
//! frontier chance nodes, so [`SubGameSolver`] runs world-partitioned safety and depth-limited
//! leaf evaluation at once. The `world` and `depth` types are re-exported, so `use subgame::*`
//! suffices.

mod depth;
mod nest;
mod world;
pub use depth::*;
pub use nest::*;
pub use world::*;

mod encoder;
mod hyperparams;
mod overlay;
mod solver;
mod unwrap;
mod wrap;

pub use encoder::*;
pub use hyperparams::*;
pub use overlay::*;
pub use solver::*;
pub use unwrap::*;
pub use wrap::*;
