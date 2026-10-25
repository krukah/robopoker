//! Mechanistic interpretability for learned abstractions.
//!
//! The solver's flop and turn buckets are 512 anonymous k-means centroids over next-street
//! equity distributions. Nothing about `F::5a` says whether it holds top pair on a dry board or
//! a naked flush draw, which makes every strategy question about it unanswerable by inspection.
mod baseline;
mod census;
mod class;
mod feature;
mod gloss;
mod glossary;
mod group;
mod profile;

pub use baseline::*;
pub use census::*;
pub use class::*;
pub use feature::*;
pub use gloss::*;
pub use glossary::*;
pub use group::*;
pub use profile::*;

#[cfg(feature = "server")]
mod lexicon;

#[cfg(feature = "server")]
pub use lexicon::*;
