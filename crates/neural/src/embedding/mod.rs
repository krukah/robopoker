//! How a network sees a game: [`Embedder`], and the rows it lays out over a net's inputs and outputs.

mod block;
mod embedder;
mod features;
mod mask;
mod target;

pub use block::*;
pub use embedder::*;
pub use features::*;
pub use mask::*;
pub use target::*;
