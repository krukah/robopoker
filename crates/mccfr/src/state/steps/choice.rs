use crate::*;

/// One choice on a path: whose turn it was, the infoset they were in, and the edge they took.
/// [`Node::decisions`] yields these walking up, [`Encoding::replay`] walking down, so reach
/// computations consume either direction identically.
pub struct Choice<F>
where
    F: CfrInfo,
{
    turn: TurnOf<F>,
    info: InfoOf<F>,
    edge: EdgeOf<F>,
}

impl<F> Clone for Choice<F>
where
    F: CfrInfo,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<F> Copy for Choice<F> where F: CfrInfo {}

impl<F> Choice<F>
where
    F: CfrInfo,
{
    pub fn new(turn: TurnOf<F>, info: InfoOf<F>, edge: EdgeOf<F>) -> Self {
        Self { turn, info, edge }
    }

    pub fn turn(&self) -> TurnOf<F> {
        self.turn
    }

    pub fn info(&self) -> &InfoOf<F> {
        &self.info
    }

    pub fn edge(&self) -> &EdgeOf<F> {
        &self.edge
    }
}
