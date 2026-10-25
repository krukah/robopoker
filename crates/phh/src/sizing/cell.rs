use super::*;
use deuce::*;

/// One `(street, depth)` cell of the bet grid, and what was played into it.
#[derive(Debug, Clone)]
pub struct Cell {
    street: Street,
    depth: usize,
    anchors: Vec<(f64, String)>,
    opening: bool,
    observed: Vec<f64>,
    shoves: usize,
}
impl Cell {
    pub fn new(
        street: Street,
        depth: usize,
        anchors: Vec<(f64, String)>,
        opening: bool,
        observed: Vec<f64>,
        shoves: usize,
    ) -> Self {
        Self {
            street,
            depth,
            anchors,
            opening,
            observed,
            shoves,
        }
    }

    pub fn street(&self) -> Street {
        self.street
    }

    /// Raises already made on this street. `MAX_RAISE_REPEATS` and above all
    /// collapse into the grid's last row.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// True when sizes here are measured in big blinds rather than pot
    /// fractions — the preflop opening row.
    pub fn opening(&self) -> bool {
        self.opening
    }

    /// The grid anchors, with the labels the grid uses for them.
    pub fn anchors(&self) -> &[(f64, String)] {
        &self.anchors
    }

    /// Observed non-shove sizes, sorted.
    pub fn observed_mut(&mut self) -> &mut Vec<f64> {
        &mut self.observed
    }

    pub fn observed(&self) -> &[f64] {
        &self.observed
    }

    /// Raises here that committed the actor's last chip. They bypass the grid
    /// entirely via `Edge::Shove`, so they are excluded from the fit.
    pub fn shoves_mut(&mut self) -> &mut usize {
        &mut self.shoves
    }

    pub fn shoves(&self) -> usize {
        self.shoves
    }

    /// Raises seen in this cell, shoves included.
    pub fn n(&self) -> usize {
        self.observed().len() + self.shoves()
    }

    /// Quantile of the observed sizes, `q` in `[0, 1]`.
    pub fn quantile(&self, q: f64) -> Option<f64> {
        self.observed()
            .get(
                ((q * (self.observed().len() as f64 - 1.0)).round() as usize)
                    .min(self.observed().len().saturating_sub(1)),
            )
            .copied()
    }

    /// Relative distance from each observed size to its nearest anchor.
    fn errors(&self) -> Vec<f64> {
        self.observed()
            .iter()
            .map(|x| {
                self.anchors()
                    .iter()
                    .map(|(a, _)| (x - a).abs() / x.abs().max(f64::EPSILON))
                    .fold(f64::INFINITY, f64::min)
            })
            .collect()
    }

    /// Median relative distance to the nearest anchor. The headline number:
    /// how much size information translation throws away in this cell.
    pub fn median_error(&self) -> Option<f64> {
        let mut errors = self.errors();
        errors.sort_by(f64::total_cmp);
        errors.get(errors.len() / 2).copied()
    }

    /// Fraction of observed sizes above the largest anchor — bets the grid
    /// can only represent by shrinking them.
    pub fn over(&self) -> f64 {
        self.beyond(|x, hi, _| x > hi)
    }

    /// Fraction of observed sizes below the smallest anchor.
    pub fn under(&self) -> f64 {
        self.beyond(|x, _, lo| x < lo)
    }

    fn beyond(&self, test: impl Fn(f64, f64, f64) -> bool) -> f64 {
        let hi = self.anchors().iter().map(|(a, _)| *a).fold(f64::NEG_INFINITY, f64::max);
        let lo = self.anchors().iter().map(|(a, _)| *a).fold(f64::INFINITY, f64::min);
        match self.observed().len() {
            0 => 0.0,
            n => self.observed().iter().filter(|x| test(**x, hi, lo)).count() as f64 / n as f64,
        }
    }

    /// Fraction of observed sizes within [`NEAR`] of some anchor.
    pub fn on_grid(&self) -> f64 {
        match self.observed().len() {
            0 => 0.0,
            n => self.errors().iter().filter(|e| **e <= NEAR).count() as f64 / n as f64,
        }
    }

    /// Share of observed sizes that would snap to each anchor, in anchor order.
    ///
    /// The sharpest read on whether the grid is spending its width where the
    /// action is: an anchor at 0% is a branch of the game tree nobody plays
    /// into, and a cell where one anchor takes nearly everything is a cell
    /// whose resolution sits in the wrong place.
    pub fn usage(&self) -> Vec<(&str, f64)> {
        let nearest = |x: &f64| {
            self.anchors()
                .iter()
                .enumerate()
                .min_by(|(_, (p, _)), (_, (q, _))| (x - p).abs().total_cmp(&(x - q).abs()))
                .map(|(index, _)| index)
        };
        let counts =
            self.observed()
                .iter()
                .filter_map(nearest)
                .fold(vec![0usize; self.anchors().len()], |mut counts, index| {
                    counts[index] += 1;
                    counts
                });
        self.anchors()
            .iter()
            .zip(counts)
            .map(|((_, label), count)| match self.observed().len() {
                0 => (label.as_str(), 0.0),
                n => (label.as_str(), count as f64 / n as f64),
            })
            .collect()
    }
}

impl Cell {}
