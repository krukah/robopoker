//! World index for safe subgame solving.
//!
//! Each world represents a quantile bucket of opponent reach probabilities,
//! used in the subgame gadget construction.
use monge::Support;

/// Index of an alternative world in the subgame gadget.
///
/// Each world represents a quantile bucket of opponent reach probabilities.
/// World 0 contains the highest-reach secrets, world K-1 the lowest.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct World(usize);

impl Support for World {}
impl World {
    pub fn index(&self) -> usize {
        self.0
    }
}

impl From<usize> for World {
    fn from(i: usize) -> Self {
        Self(i)
    }
}
