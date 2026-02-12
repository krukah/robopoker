//! Edge type for subgame-augmented games.
use crate::*;
use rbp_transport::*;

/// Edge type for subgame-augmented games.
///
/// Wraps the inner game's edge type and adds the ability to select
/// alternatives at the subgame phase.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubEdge<E>
where
    E: CfrEdge,
{
    /// Inner game action.
    Inner(E),
    /// Selection of alternative index at subgame phase.
    World(usize),
}

impl<E> Support for SubEdge<E> where E: CfrEdge {}
impl<E> CfrEdge for SubEdge<E> where E: CfrEdge {}
