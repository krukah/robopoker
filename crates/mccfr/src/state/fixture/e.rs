use crate::*;
use monge::Support;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct E(pub u8);

impl Support for E {}

impl CfrEdge for E {}
