use crate::*;
use mccfr::*;

/// A world-conditioned info set: the inner public half tagged with its sampled world, the inner
/// secret unchanged.
pub type WorldInfo<I> = Composite<WorldPublic<<I as CfrInfo>::X>, <I as CfrInfo>::Y>;
