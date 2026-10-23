use crate::*;
use mccfr::*;

/// A nested info set: the inner public half tagged by whether it sits at the off-tree entry,
/// below it, or on the canonical tree; the inner secret unchanged.
pub type NestInfo<I, Off> = Composite<NestPublic<<I as CfrInfo>::X, Off>, <I as CfrInfo>::Y>;
