//! Deep CFR's networks in candle: any [`Architecture`] on any [`Backend`], over any
//! [`Embedder`](crate::Embedder).

mod architecture;
mod backend;
mod cpu;
mod fitted;
mod head;
mod minibatch;
mod mlp;
mod net;
mod trainer;

pub use architecture::*;
pub use backend::*;
pub use cpu::*;
pub use fitted::*;
pub use head::*;
pub use minibatch::*;
pub use mlp::*;
pub use net::*;
pub use trainer::*;
