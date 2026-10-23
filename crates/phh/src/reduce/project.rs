#![doc = include_str!("../doc/project.md")]

use crate::*;
use deuce::*;

/// Namespace for the project routines.
pub struct Project;

impl Project {
    /// Classifies a replayed hand.
    ///
    /// Needs the [`Outcome`] rather than just the [`Record`] because who was still
    /// live at the flop is a replay fact, not a syntactic one.
    pub fn shape(record: &Record, outcome: &Outcome) -> Shape {
        let blinds = record.blinds();
        match outcome.flopped() {
            [] | [_] => Shape::Preflop,
            [a, b] => {
                let forced = |i: usize| blinds.get(i).copied().unwrap_or(0) > 0;
                // Heads-up already has the right order; only a multiway pot can
                // strand both blinds together and invert it.
                if record.n() > 2 && forced(*a) && forced(*b) {
                    Shape::Inverted
                } else {
                    Shape::HeadsUp(Seats::new(*a, *b))
                }
            }
            many => Shape::Multiway(many.len()),
        }
    }

    /// Extracts the projection inputs for a hand that reduces to our tree.
    ///
    /// Returns `None` for shapes that do not reduce.
    pub fn reduce(record: &Record, outcome: &Outcome) -> Option<Reduction> {
        let seats = Project::shape(record, outcome).seats()?;
        let preflop = outcome
            .nodes()
            .iter()
            .take_while(|n| n.street() == Street::Pref)
            .collect::<Vec<_>>();
        let flop = outcome.nodes().iter().find(|n| n.street() == Street::Flop)?;
        let contributed = |seat: usize| {
            preflop
                .iter()
                .filter(|n| n.seat() == seat)
                .map(|n| n.stake() + n.put())
                .max()
                .unwrap_or_else(|| record.blinds().get(seat).copied().unwrap_or(0))
        };
        Some(Reduction::new(
            seats,
            flop.pot(),
            flop.pot() - contributed(seats.oop()) - contributed(seats.ip()),
            preflop.iter().filter(|n| matches!(n.step(), Step::BetTo(_, _))).count(),
            preflop
                .iter()
                .rev()
                .find(|n| matches!(n.step(), Step::BetTo(_, _)))
                .copied()
                .map(Node::seat),
            record
                .starting_stacks()
                .get(seats.oop())
                .copied()
                .unwrap_or(0)
                .saturating_sub(contributed(seats.oop()))
                .min(
                    record
                        .starting_stacks()
                        .get(seats.ip())
                        .copied()
                        .unwrap_or(0)
                        .saturating_sub(contributed(seats.ip())),
                ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    impl Project {
        fn hand(actions: &str) -> (Record, Outcome) {
            let text = format!(
                "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
                 blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
                 starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
                 actions = [{DEAL}, {actions}]\n"
            );
            let record = crate::Parse::parse(&text, "test").unwrap();
            let outcome = crate::Replay::replay(&record).unwrap();
            (record, outcome)
        }
    }

    #[test]
    fn folded_around_is_preflop() {
        let (r, o) = Project::hand("'p3 f', 'p4 f', 'p5 f', 'p6 f', 'p1 f'");
        assert_eq!(Project::shape(&r, &o), Shape::Preflop);
        assert!(Project::reduce(&r, &o).is_none());
    }

    #[test]
    fn blind_versus_blind_is_inverted() {
        let (r, o) = Project::hand("'p3 f', 'p4 f', 'p5 f', 'p6 f', 'p1 cc', 'p2 cc', 'd db 7d5h9d', 'p1 cc', 'p2 cc'");
        assert_eq!(Project::shape(&r, &o), Shape::Inverted);
        assert!(Project::reduce(&r, &o).is_none());
    }

    #[test]
    fn big_blind_versus_a_late_seat_reduces() {
        let (r, o) =
            Project::hand("'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', 'd db 7d5h9d', 'p2 cc', 'p6 cc'");
        let seats = Project::shape(&r, &o).seats().expect("reduces");
        assert_eq!(seats.oop(), 1);
        assert_eq!(seats.ip(), 5);
        let reduction = Project::reduce(&r, &o).expect("reduces");
        // 225 from the button, 225 from the big blind, 50 dead from the small.
        assert_eq!(reduction.pot(), 500);
        assert_eq!(reduction.dead(), 50);
        assert_eq!(reduction.pot_bb(100), 5.0);
        assert_eq!(reduction.depth(), 1);
        assert!(reduction.ip_is_aggressor());
        assert_eq!(reduction.effective_bb(100), 97.75);
    }

    #[test]
    fn three_to_the_flop_is_multiway() {
        let (r, o) = Project::hand(
            "'p3 f', 'p4 f', 'p5 cbr 225', 'p6 cc', 'p1 f', 'p2 cc', \
             'd db 7d5h9d', 'p2 cc', 'p5 cc', 'p6 cc'",
        );
        assert_eq!(Project::shape(&r, &o), Shape::Multiway(3));
    }
}
