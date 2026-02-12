//! Iterator over waypoints in a trajectory.
use super::trajectory::Trajectory;
use super::waypoint::Waypoint;
use crate::*;

/// Iterator over waypoints in a trajectory.
pub struct Waypoints<'path, G, E>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
{
    pub(crate) trajectory: &'path Trajectory<G, E>,
    pub(crate) state: G,
    pub(crate) index: usize,
}

impl<'path, G, E> Iterator for Waypoints<'path, G, E>
where
    G: CfrGame<E = E>,
    E: CfrEdge,
{
    type Item = Waypoint<'path, G, E>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.trajectory.path().len() {
            return None;
        }
        let waypoint = Waypoint {
            state: self.state,
            past: &self.trajectory.path()[..self.index],
            edge: self.trajectory.path()[self.index],
        };
        self.state = self.state.apply(self.trajectory.path()[self.index]);
        self.index += 1;
        Some(waypoint)
    }
}
