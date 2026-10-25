//! Deep CFR on Kuhn and RPS in the browser: the `neural` training loop, compiled to WebAssembly.
//! See `README.md`.

#[path = "../../neural/src/network/kuhn.rs"]
mod kuhnembed;
#[path = "../../neural/src/network/rps.rs"]
mod rpsembed;

mod frame;
mod knobs;
mod lab;
mod lane;
mod probe;
mod reading;
mod run;
mod session;

pub use frame::*;
pub use knobs::*;
pub use kuhnembed::*;
pub use lab::*;
pub use lane::*;
pub use probe::*;
pub use reading::*;
pub use rpsembed::*;
pub use run::*;
pub use session::*;

use neural::*;
