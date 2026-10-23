use mccfr::*;
use monge::Support;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum MiniEdge {
    Bet,
    Check,
}

impl Support for MiniEdge {}

impl CfrEdge for MiniEdge {}
