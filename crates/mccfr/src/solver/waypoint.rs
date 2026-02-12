//! A waypoint along a trajectory — a decision point with context.
use crate::*;

/// A waypoint along a trajectory — a decision point with context.
///
/// References the trajectory's path slice to avoid allocation.
/// State and edge are `Copy` so owned directly.
#[derive(Debug, Clone, Copy)]
pub struct Waypoint<'path, G, E>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
{
    /// Game state at this decision point.
    pub state: G,
    /// Path taken to reach this state (slice into trajectory).
    pub past: &'path [E],
    /// Edge that was taken at this decision.
    pub edge: E,
}
