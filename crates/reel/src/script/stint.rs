use crate::*;

/// One phase of the scene in flat-layer indices: where its board layer sits,
/// where its decision layers sit, and the chosen slot at each.
#[derive(Debug, Clone)]
pub struct Stint {
    deal: usize,
    moves: Vec<usize>,
    picks: Vec<Slot>,
}

impl Stint {
    pub fn new(deal: usize) -> Self {
        Self {
            deal,
            moves: Vec::new(),
            picks: Vec::new(),
        }
    }

    pub fn push(&mut self, layer: usize, pick: Slot) {
        self.moves_mut().push(layer);
        self.picks_mut().push(pick);
    }

    pub fn deal(&self) -> usize {
        self.deal
    }

    pub fn moves(&self) -> &[usize] {
        &self.moves
    }

    pub fn picks(&self) -> &[Slot] {
        &self.picks
    }

    fn moves_mut(&mut self) -> &mut Vec<usize> {
        &mut self.moves
    }

    fn picks_mut(&mut self) -> &mut Vec<Slot> {
        &mut self.picks
    }
}
