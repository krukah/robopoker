//! Tier 1: is our action abstraction the one a strong agent actually bets?
//!
//! Needs no blueprint and no engine replay — only [`Node`]s and the live grid constants. It
//! answers a question that is upstream of every strategy metric: if the sizes we trained on are
//! not the sizes that get played, translation eats the difference before the policy is ever
//! consulted.

use crate::*;
use deuce::*;
use kicker::*;
use pokerkit::*;

/// Relative distance within which an observed size counts as "on grid".
pub const NEAR: f64 = 0.15;

/// Namespace for the calibrate routines.
pub struct Calibrate;

impl Calibrate {
    /// Measures every raise in `nodes` against the live bet grid.
    ///
    /// `bblind` is the big blind at the log's own chip scale, used only for the
    /// preflop opening axis.
    pub fn calibrate<'nodes>(nodes: impl IntoIterator<Item = &'nodes Node>, bblind: Amount) -> Calibration {
        let mut cells = Street::all()
            .into_iter()
            .flat_map(|street| (0..=MAX_RAISE_REPEATS.min(2)).map(move |depth| Calibrate::cell(street, depth)))
            .collect::<Vec<_>>();
        nodes
            .into_iter()
            .filter(|node| matches!(node.step(), Step::BetTo(_, _)))
            .for_each(|node| {
                let depth = node.depth().min(2);
                let Some(cell) = cells
                    .iter_mut()
                    .find(|c| c.street() == node.street() && c.depth() == depth)
                else {
                    return;
                };
                match node.is_shove() {
                    true => *cell.shoves_mut() += 1,
                    false if cell.opening() => cell.observed_mut().push(node.bb_units(bblind)),
                    false => cell.observed_mut().push(node.pot_fraction()),
                }
            });
        cells.iter_mut().for_each(|c| c.observed_mut().sort_by(f64::total_cmp));
        Calibration::new(cells)
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
        Cell::new(street, depth, anchors, opening, Vec::new(), 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    impl Calibrate {
        fn nodes(actions: &str) -> Vec<Node> {
            let text = format!(
                "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
                 blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
                 starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
                 actions = [{DEAL}, {actions}]\n"
            );
            crate::Replay::replay(&crate::Parse::parse(&text, "test").unwrap())
                .unwrap()
                .nodes()
                .to_vec()
        }
    }

    impl Calibration {
        fn at(&self, street: Street, depth: usize) -> &Cell {
            self.cells()
                .iter()
                .find(|c| c.street() == street && c.depth() == depth)
                .unwrap()
        }
    }

    #[test]
    fn preflop_opens_measure_in_big_blinds() {
        let calibration =
            Calibrate::calibrate(Calibrate::nodes("'p3 cbr 225', 'p4 f', 'p5 f', 'p6 f', 'p1 f', 'p2 f'").iter(), 100);
        let cell = calibration.at(Street::Pref, 0);
        assert!(cell.opening());
        assert_eq!(cell.observed(), [2.25]);
        // OPENS anchors at 2bb and 3bb straddle 2.25; nearest is 2bb, and the
        // error is relative to what was played: 0.25 / 2.25.
        assert_eq!(cell.median_error(), Some(0.25 / 2.25));
        assert_eq!(cell.over(), 0.0);
    }

    #[test]
    fn postflop_bets_measure_in_pot_fractions() {
        let calibration = Calibrate::calibrate(
            Calibrate::nodes(
                "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
                 'd db 7d5h9d', 'p2 cbr 250', 'p6 f'",
            )
            .iter(),
            100,
        );
        let cell = calibration.at(Street::Flop, 0);
        assert!(!cell.opening());
        // 250 into a 500 pot is exactly the 1/2 anchor.
        assert_eq!(cell.observed(), [0.5]);
        assert_eq!(cell.median_error(), Some(0.0));
        assert_eq!(cell.on_grid(), 1.0);
        assert_eq!(cell.usage(), [("1/4", 0.0), ("1/2", 1.0), ("1/1", 0.0), ("2/1", 0.0)]);
    }

    #[test]
    fn a_raise_includes_the_call_in_its_numerator() {
        let calibration = Calibrate::calibrate(
            Calibrate::nodes(
                "'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', \
                 'd db 7d5h9d', 'p2 cbr 250', 'p6 cbr 1000', 'p2 f'",
            )
            .iter(),
            100,
        );
        let cell = calibration.at(Street::Flop, 1);
        // Pot is 750 when the button acts; it puts in the full 1000.
        assert_eq!(cell.observed(), [4.0 / 3.0]);
        assert_eq!(cell.over(), 1.0);
    }

    #[test]
    fn shoves_are_counted_but_not_fitted() {
        let calibration = Calibrate::calibrate(
            Calibrate::nodes("'p3 cbr 10000', 'p4 f', 'p5 f', 'p6 f', 'p1 f', 'p2 f'").iter(),
            100,
        );
        let cell = calibration.at(Street::Pref, 0);
        assert_eq!(cell.shoves(), 1);
        assert!(cell.observed().is_empty());
        assert_eq!(cell.n(), 1);
    }
}
