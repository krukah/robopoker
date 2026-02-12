//! Private state for subgame-augmented games.
use crate::*;
use rbp_transport::Support;

/// Private component for subgame info.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubSecret<Y>
where
    Y: CfrSecret,
{
    Inner(Y),
    Root,
}

impl<Y> Support for SubSecret<Y> where Y: CfrSecret {}
impl<Y> CfrSecret for SubSecret<Y> where Y: CfrSecret {}
