#![cfg(feature = "server")]

//! Autotrain module - unified training pipeline
//!
//! Manages the complete training workflow:
//! 1. Check current state in postgres
//! 2. Run clustering for incomplete streets
//! 3. Run blueprint training

mod epoch;
mod fast;
mod mode;
mod pretraining;
mod slow;
mod trainer;

pub use epoch::*;
pub use fast::*;
pub use mode::*;
pub use pretraining::*;
pub use slow::*;
pub use trainer::*;
