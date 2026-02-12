/// Marker trait for types that form the support of a probability distribution.
///
/// In measure theory, the support of a distribution is the smallest closed set
/// containing all points with positive probability. This trait marks types that
/// can serve as elements of such a support set.
///
/// The `Clone` bound enables copying support elements when constructing
/// transport plans and iterating over distributions.
pub trait Support: Clone {}

/// usize implements Support for use as world indices in subgame solving.
impl Support for usize {}
