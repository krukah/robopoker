use crate::*;
use pokerkit::Probability;

/// A probability distribution over a game's secret space, charted on the
/// [`Atlas`] grid — the generic form of the range panel's posterior. The
/// 169-hand `RangePosterior` is this type instantiated at NLHE.
#[derive(Debug, Clone, PartialEq)]
pub struct Posterior<G>
where
    G: Atlas,
{
    raw: Range<G, Probability>,
    peak: Probability,
}

impl<G> Default for Posterior<G>
where
    G: Atlas,
{
    fn default() -> Self {
        Self::from_raw(vec![0.0; G::size()])
    }
}

impl<G> Posterior<G>
where
    G: Atlas,
{
    /// Wraps raw per-cell mass, computing the display peak.
    pub fn from_raw(raw: Vec<Probability>) -> Self {
        Self {
            peak: raw.iter().copied().fold(0.0, Probability::max),
            raw: Range::from_cells(raw),
        }
    }

    /// The uniform prior over every cell.
    pub fn uniform() -> Self {
        Self::from_raw(vec![1.0 / G::size() as Probability; G::size()])
    }

    /// Flat index of a grid cell.
    pub fn index(row: usize, col: usize) -> usize {
        Range::<G, Probability>::index(row, col)
    }

    /// Display-normalized weight in [0, 1] — heatmap opacity.
    pub fn weight(&self, row: usize, col: usize) -> Probability {
        if self.peak() > 0.0 { self.raw_weight(row, col) / self.peak() } else { 0.0 }
    }

    /// Raw posterior mass for the cell — hover labels.
    pub fn raw_weight(&self, row: usize, col: usize) -> Probability {
        *self.raw().at(row, col)
    }

    fn raw(&self) -> &Range<G, Probability> {
        &self.raw
    }

    fn peak(&self) -> Probability {
        self.peak
    }
}

impl<G> FromIterator<(usize, Probability)> for Posterior<G>
where
    G: Atlas,
{
    /// Scatter-accumulates (cell index, mass) pairs onto the grid —
    /// repeated indices sum, so suit isomorphisms collapse naturally.
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (usize, Probability)>,
    {
        Self::from_raw(iter.into_iter().fold(vec![0.0; G::size()], |mut raw, (index, mass)| {
            raw[index] += mass;
            raw
        }))
    }
}
