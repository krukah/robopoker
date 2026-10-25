/// The index of a tile within a layer — an edge among a seat's options or a
/// card among a street's reveals. Newtyped so it can never be confused with
/// layer indices, depths, or any other bare usize crossing a boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot(usize);

impl Slot {
    pub fn new(at: usize) -> Self {
        Self(at)
    }

    pub fn raw(&self) -> usize {
        self.0
    }
}
