//! Tier 1: is our action abstraction the one a strong agent actually bets?
//!
//! Needs no blueprint and no engine replay — only [`Node`]s and the live grid
//! constants. It answers a question that is upstream of every strategy metric:
//! if the sizes we trained on are not the sizes that get played, translation
//! eats the difference before the policy is ever consulted.
//!
//! Sizes are measured the way `kicker` measures them, not the way a poker
//! player would. `Action::Raise(chips)` is an increment from the actor's
//! current stake, denominated against `Game::pot()` *before* the action, and
//! `Edge::Open(n)` puts in `n` big blinds. A raise facing a bet therefore
//! includes the call in its numerator and excludes it from its denominator.
//! Matching that convention exactly is the whole point — the numbers have to
//! describe the anchors `Size::translate` will actually pick between.
//!
//! Anchors come from [`kicker::Size::grid`], so this tracks `RAISES`, `OPENS`
//! and `PLURIBUS_INDICES` as they change rather than restating them.

use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// Relative distance within which an observed size counts as "on grid".
pub const NEAR: f64 = 0.15;

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
    pub fn observed(&self) -> &[f64] {
        &self.observed
    }

    /// Raises here that committed the actor's last chip. They bypass the grid
    /// entirely via `Edge::Shove`, so they are excluded from the fit.
    pub fn shoves(&self) -> usize {
        self.shoves
    }

    /// Raises seen in this cell, shoves included.
    pub fn n(&self) -> usize {
        self.observed.len() + self.shoves
    }

    /// Quantile of the observed sizes, `q` in `[0, 1]`.
    pub fn quantile(&self, q: f64) -> Option<f64> {
        self.observed
            .get(((q * (self.observed.len() as f64 - 1.0)).round() as usize).min(self.observed.len().saturating_sub(1)))
            .copied()
    }

    /// Relative distance from each observed size to its nearest anchor.
    fn errors(&self) -> Vec<f64> {
        self.observed
            .iter()
            .map(|x| {
                self.anchors
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
        let hi = self.anchors.iter().map(|(a, _)| *a).fold(f64::NEG_INFINITY, f64::max);
        let lo = self.anchors.iter().map(|(a, _)| *a).fold(f64::INFINITY, f64::min);
        match self.observed.len() {
            0 => 0.0,
            n => self.observed.iter().filter(|x| test(**x, hi, lo)).count() as f64 / n as f64,
        }
    }

    /// Fraction of observed sizes within [`NEAR`] of some anchor.
    pub fn on_grid(&self) -> f64 {
        match self.observed.len() {
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
            self.anchors
                .iter()
                .enumerate()
                .min_by(|(_, (p, _)), (_, (q, _))| (x - p).abs().total_cmp(&(x - q).abs()))
                .map(|(index, _)| index)
        };
        let counts =
            self.observed
                .iter()
                .filter_map(nearest)
                .fold(vec![0usize; self.anchors.len()], |mut counts, index| {
                    counts[index] += 1;
                    counts
                });
        self.anchors
            .iter()
            .zip(counts)
            .map(|((_, label), count)| match self.observed.len() {
                0 => (label.as_str(), 0.0),
                n => (label.as_str(), count as f64 / n as f64),
            })
            .collect()
    }
}

/// The full grid, cell by cell.
#[derive(Debug, Clone)]
pub struct Calibration {
    cells: Vec<Cell>,
}

impl Calibration {
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Total raises measured.
    pub fn n(&self) -> usize {
        self.cells.iter().map(Cell::n).sum()
    }
}

/// Measures every raise in `nodes` against the live bet grid.
///
/// `bblind` is the big blind at the log's own chip scale, used only for the
/// preflop opening axis.
pub fn calibrate<'a>(nodes: impl IntoIterator<Item = &'a Node>, bblind: Amount) -> Calibration {
    let mut cells = Street::all()
        .into_iter()
        .flat_map(|street| (0..=MAX_RAISE_REPEATS.min(2)).map(move |depth| cell(street, depth)))
        .collect::<Vec<_>>();
    nodes
        .into_iter()
        .filter(|node| matches!(node.step(), Step::BetTo(_, _)))
        .for_each(|node| {
            let depth = node.depth().min(2);
            let Some(cell) = cells.iter_mut().find(|c| c.street == node.street() && c.depth == depth) else {
                return;
            };
            match node.is_shove() {
                true => cell.shoves += 1,
                false if cell.opening => cell.observed.push(node.bb_units(bblind)),
                false => cell.observed.push(node.pot_fraction()),
            }
        });
    cells.iter_mut().for_each(|c| c.observed.sort_by(f64::total_cmp));
    Calibration { cells }
}

/// Builds an empty cell carrying the anchors the engine would offer there.
fn cell(street: Street, depth: usize) -> Cell {
    let (anchors, opening) = match Size::grid(street, depth) {
        Some(Grid::Opening(bbs)) => (bbs.iter().map(|n| (f64::from(*n), format!("{n}bb"))).collect(), true),
        Some(Grid::Postflop(idx)) => (
            idx.iter()
                .map(|i| RAISES[*i])
                .map(|(n, d)| (f64::from(n) / f64::from(d), format!("{n}/{d}")))
                .collect(),
            false,
        ),
        None => (Vec::new(), false),
    };
    Cell {
        street,
        depth,
        anchors,
        opening,
        observed: Vec::new(),
        shoves: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    fn nodes(actions: &str) -> Vec<Node> {
        let text = format!(
            "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
             blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
             starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
             actions = [{DEAL}, {actions}]\n"
        );
        crate::replay(&crate::parse(&text, "test").unwrap())
            .unwrap()
            .nodes()
            .to_vec()
    }

    fn cell_of(c: &Calibration, street: Street, depth: usize) -> &Cell {
        c.cells()
            .iter()
            .find(|c| c.street() == street && c.depth() == depth)
            .unwrap()
    }

    #[test]
    fn preflop_opens_measure_in_big_blinds() {
        let calibration = calibrate(nodes("'p3 cbr 225', 'p4 f', 'p5 f', 'p6 f', 'p1 f', 'p2 f'").iter(), 100);
        let cell = cell_of(&calibration, Street::Pref, 0);
        assert!(cell.opening());
        assert_eq!(cell.observed(), [2.25]);
        // OPENS anchors at 2bb and 3bb straddle 2.25; nearest is 2bb, and the
        // error is relative to what was played: 0.25 / 2.25.
        assert_eq!(cell.median_error(), Some(0.25 / 2.25));
        assert_eq!(cell.over(), 0.0);
    }

    #[test]
    fn postflop_bets_measure_in_pot_fractions() {
        let calibration = calibrate(
            nodes(
                "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
                 'd db 7d5h9d', 'p2 cbr 250', 'p6 f'",
            )
            .iter(),
            100,
        );
        let cell = cell_of(&calibration, Street::Flop, 0);
        assert!(!cell.opening());
        // 250 into a 500 pot is exactly the 1/2 anchor.
        assert_eq!(cell.observed(), [0.5]);
        assert_eq!(cell.median_error(), Some(0.0));
        assert_eq!(cell.on_grid(), 1.0);
        assert_eq!(cell.usage(), [("1/4", 0.0), ("1/2", 1.0), ("3/4", 0.0), ("1/1", 0.0), ("2/1", 0.0)]);
    }

    #[test]
    fn a_raise_includes_the_call_in_its_numerator() {
        let calibration = calibrate(
            nodes(
                "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
                 'd db 7d5h9d', 'p2 cbr 250', 'p6 cbr 1000', 'p2 f'",
            )
            .iter(),
            100,
        );
        let cell = cell_of(&calibration, Street::Flop, 1);
        // Pot is 750 when the button acts; it puts in the full 1000.
        assert_eq!(cell.observed(), [4.0 / 3.0]);
        assert_eq!(cell.over(), 1.0);
    }

    #[test]
    fn shoves_are_counted_but_not_fitted() {
        let calibration = calibrate(nodes("'p3 cbr 10000', 'p4 f', 'p5 f', 'p6 f', 'p1 f', 'p2 f'").iter(), 100);
        let cell = cell_of(&calibration, Street::Pref, 0);
        assert_eq!(cell.shoves(), 1);
        assert!(cell.observed().is_empty());
        assert_eq!(cell.n(), 1);
    }
}
