use crate::*;

/// Convenience supertrait combining all profile capabilities for training.
///
/// Requires [`CfrFlow`] (CFR math, blanket from Profile + CfrSampling),
/// [`CfrNash`] (Nash queries, blanket from Profile), and `Storage`
/// (mutable write access). All share associated types via [`CfrRule`].
pub trait CfrSolution: CfrFlow + MutProf {}
impl<T> CfrSolution for T where T: CfrFlow + MutProf {}
