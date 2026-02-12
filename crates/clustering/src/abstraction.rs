//! Clustering-specific wrapper for Abstraction with Support trait.
use rbp_gameplay::Abstraction;
use rbp_transport::Support;

/// Newtype wrapper for Abstraction that implements Support.
/// Used for optimal transport calculations in clustering.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClusterAbs(Abstraction);

impl Support for ClusterAbs {}

impl From<Abstraction> for ClusterAbs {
    fn from(abs: Abstraction) -> Self {
        Self(abs)
    }
}
impl From<ClusterAbs> for Abstraction {
    fn from(wrapper: ClusterAbs) -> Self {
        wrapper.0
    }
}
impl std::ops::Deref for ClusterAbs {
    type Target = Abstraction;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
