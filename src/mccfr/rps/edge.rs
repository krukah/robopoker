use crate::mccfr::*;
use crate::transport::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RpsEdge {
    R,
    P,
    S,
}

impl Support for RpsEdge {}
impl TreeEdge for RpsEdge {}
