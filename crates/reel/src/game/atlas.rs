use crate::*;
use pokerkit::Probability;

/// The secret-space chart: how a game's private information arranges into a
/// 2D grid of cells. The 169-hand range grid is the NLHE implementation of
/// this trait; Kuhn's is a single row of three ranks. Weights are the demo's
/// deterministic synthetic posterior until Profiled walks land (stage 4).
pub trait Atlas: Scenic {
    /// Grid dimensions (columns, rows).
    fn grid() -> (usize, usize);

    /// Number of cells on the chart.
    fn size() -> usize {
        Self::grid().0 * Self::grid().1
    }

    /// The label of the cell at (col, row).
    fn cell(col: usize, row: usize) -> String;

    /// The demo posterior weight of a cell at phase `s`, for the hero's
    /// representing range or the villain's defending range.
    fn weight(col: usize, row: usize, phase: usize, hero: bool) -> Probability;

    /// The hero's own holding, highlighted on the hero sheet.
    fn spot() -> (usize, usize);

    /// The demo's utility distributions at a phase: (observation,
    /// abstraction) bucket series for the equity curve. Deterministic
    /// synthetic story data — the reel is authored, never sampled.
    fn curves(phase: usize) -> (Vec<f64>, Vec<f64>);
}
