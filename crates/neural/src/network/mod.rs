//! The two function approximators Deep CFR trains, and [`MockNet`], a tabular reference for both.

#[cfg(test)]
mod counted;
#[cfg(test)]
mod kuhn;
mod mock;
mod regret;
#[cfg(test)]
mod rps;
mod strategy;

#[cfg(test)]
pub use counted::*;
#[cfg(test)]
pub use kuhn::*;
pub use mock::*;
pub use regret::*;
#[cfg(test)]
pub use rps::*;
pub use strategy::*;
