//! Edge type for subgame-augmented games.
use crate::*;
use rbp_core::Probability;
use rbp_transport::*;

/// Fixed continuation policies available at a depth-limited frontier.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Continuation {
    Blueprint,
    FoldBiased,
    CallBiased,
    RaiseBiased,
}

impl Continuation {
    pub const ALL: [Self; 4] = [
        Self::Blueprint,
        Self::FoldBiased,
        Self::CallBiased,
        Self::RaiseBiased,
    ];
    const BIAS: Probability = 5.0;

    pub fn multiplier<E: CfrEdge>(&self, edge: &E) -> Probability {
        match self {
            Self::Blueprint => 1.0,
            Self::FoldBiased if edge.is_fold() => Self::BIAS,
            Self::CallBiased if edge.is_call() => Self::BIAS,
            Self::RaiseBiased if edge.is_raise() => Self::BIAS,
            _ => 1.0,
        }
    }
}

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
    /// Selection of a fixed continuation policy at a depth-limited frontier.
    Continuation(Continuation),
}

impl<E> Support for SubEdge<E> where E: CfrEdge {}
impl<E> CfrEdge for SubEdge<E> where E: CfrEdge {}
