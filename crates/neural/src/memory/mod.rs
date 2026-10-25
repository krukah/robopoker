//! The reservoirs `M_V,p` and `M_Π`, the rows they hold, and the [`Memories`] that group them.

mod memories;
mod reservoir;
mod sample;

pub use memories::*;
pub use reservoir::*;
pub use sample::*;
