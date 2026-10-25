use super::*;

/// The full grid, cell by cell.
#[derive(Debug, Clone)]
pub struct Calibration {
    cells: Vec<Cell>,
}
impl Calibration {
    pub fn new(cells: Vec<Cell>) -> Self {
        Self { cells }
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Total raises measured.
    pub fn n(&self) -> usize {
        self.cells().iter().map(Cell::n).sum()
    }
}

impl Calibration {}
