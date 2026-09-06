use crate::*;

/// Supertrait combining every profile capability needed for training: CFR math
/// via [`CfrFlow`] and write access via [`MutProf`].
pub trait CfrSolution: CfrFlow + MutProf {}
impl<T> CfrSolution for T where T: CfrFlow + MutProf {}
