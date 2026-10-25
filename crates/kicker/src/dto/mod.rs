//! Data transfer objects for API communication.
mod gameplay;
mod request;
mod strategy;
mod topology;
mod training;
pub use gameplay::*;
pub(crate) use request::*;
pub use strategy::*;
pub use topology::*;
pub use training::*;
