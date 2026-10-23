use crate::*;

/// One decision along the authored demonstration: the concrete state, the
/// real menu of legal actions there, and the slot the line takes.
#[derive(Debug, Clone)]
pub struct Step<G>
where
    G: Scenic,
{
    state: G,
    menu: Vec<G::E>,
    pick: Slot,
}

impl<G> Step<G>
where
    G: Scenic,
{
    pub fn new(state: G, menu: Vec<G::E>, pick: Slot) -> Self {
        Self { state, menu, pick }
    }

    pub fn state(&self) -> &G {
        &self.state
    }

    pub fn menu(&self) -> &[G::E] {
        &self.menu
    }

    pub fn pick(&self) -> Slot {
        self.pick
    }
}
