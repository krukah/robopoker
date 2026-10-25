use crate::*;

/// A game-shaped collection: one value for every cell in an [`Atlas`] grid.
///
/// [`crate::Posterior`] is one specialization (`Probability` per cell); strategy,
/// equity, and diagnostics can all use the same `Hole → T` surface without
/// re-inventing indexing or grid allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct Range<G, T>
where
    G: Atlas,
{
    cells: Vec<T>,
    game: std::marker::PhantomData<G>,
}

impl<G, T> Range<G, T>
where
    G: Atlas,
{
    pub fn from_cells(cells: Vec<T>) -> Self {
        assert_eq!(cells.len(), G::size(), "range must fill its atlas");
        Self {
            cells,
            game: std::marker::PhantomData,
        }
    }

    pub fn from_fn(mut build: impl FnMut(usize, usize) -> T) -> Self {
        Self::from_cells(
            (0..G::size())
                .map(|index| build(index / G::grid().0, index % G::grid().0))
                .collect(),
        )
    }

    pub fn index(row: usize, col: usize) -> usize {
        row * G::grid().0 + col
    }

    pub fn at(&self, row: usize, col: usize) -> &T {
        &self.cells()[Self::index(row, col)]
    }

    pub fn at_mut(&mut self, row: usize, col: usize) -> &mut T {
        &mut self.cells_mut()[Self::index(row, col)]
    }

    pub fn cells(&self) -> &[T] {
        &self.cells
    }

    fn cells_mut(&mut self) -> &mut [T] {
        &mut self.cells
    }

    pub fn into_cells(self) -> Vec<T> {
        self.cells
    }
}
