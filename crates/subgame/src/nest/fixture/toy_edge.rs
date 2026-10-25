use mccfr::*;
use monge::Support;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToyEdge {
    Check,
    Bet,
}

impl Support for ToyEdge {}

impl CfrEdge for ToyEdge {}
