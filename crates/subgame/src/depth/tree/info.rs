use crate::*;
use mccfr::*;

/// A depth-limited info set: the inner public half tagged with its phase, the inner secret
/// unchanged.
pub type DepthInfo<I, const D: usize> = Composite<DepthPublic<<I as CfrInfo>::X, D>, <I as CfrInfo>::Y>;
