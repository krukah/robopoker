#[cfg(feature = "server")]
mod tables;
#[cfg(feature = "server")]
pub use tables::*;

#[cfg(feature = "database")]
mod postgres;
#[cfg(feature = "database")]
pub use postgres::*;

#[cfg(feature = "disk")]
#[deprecated]
mod disk;
#[cfg(feature = "disk")]
#[allow(deprecated)]
pub use disk::*;
