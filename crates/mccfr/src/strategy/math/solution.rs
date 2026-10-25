use crate::*;

/// Supertrait combining every profile capability needed for training: CFR math
/// via [`Flow`] and write access via [`Storage`].
pub trait Solution: Flow + Storage {}
impl<T> Solution for T where T: Flow + Storage {}
