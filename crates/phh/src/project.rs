//! Classifying a multiway hand against our heads-up game tree.
//!
//! Our blueprint is `pokerkit::N = 2`. A 6-max log is not that game, but most of
//! it collapses onto that game: ~90% of Pluribus pots that see a flop see it
//! heads-up. The question is which of those collapse *faithfully*.
//!
//! The discriminator is postflop action order. In 6-max, betting opens left of
//! the button, so the small blind acts first on every postflop street. In
//! heads-up the small blind **is** the button and acts last. So:
//!
//! - **blind vs. blind** — SB acts first postflop, BB last. Heads-up has it the
//!   other way round. Not our game; [`Shape::Inverted`].
//! - **anything else** — the earlier seat acts first postflop and the later seat
//!   last, which is exactly the heads-up shape with the earlier seat playing the
//!   big blind's role and the later seat playing the button's.
//!   [`Shape::HeadsUp`].
//!
//! What the reduction does not preserve is the pot: a folded blind leaves dead
//! money our tree cannot represent (0.5bb in 2,471 Pluribus hands, 1bb in 521,
//! 1.5bb in 605). [`Reduction`] carries the flop pot in big blinds so the caller
//! can project onto the nearest reachable heads-up preflop path and absorb the
//! difference there.

use crate::*;
use deuce::*;

/// The two seats of a reducible pot, in heads-up roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seats {
    oop: usize,
    ip: usize,
}

impl Seats {
    /// Acts first postflop — the big blind's role in our tree.
    pub fn oop(&self) -> usize {
        self.oop
    }

    /// Acts last postflop — the button's role in our tree.
    pub fn ip(&self) -> usize {
        self.ip
    }

    /// True if `seat` is one of the two.
    pub fn has(&self, seat: usize) -> bool {
        self.oop == seat || self.ip == seat
    }
}

/// How a logged hand relates to our heads-up tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// Ended before a flop. No postflop decisions to compare.
    Preflop,
    /// Three or more players saw the flop. Outside our tree entirely.
    Multiway(usize),
    /// Heads-up on the flop, but blind vs. blind, so postflop action order is
    /// inverted relative to heads-up. Unusable.
    Inverted,
    /// Heads-up on the flop, in heads-up action order.
    HeadsUp(Seats),
}

impl Shape {
    /// The two seats, when the hand reduces.
    pub fn seats(&self) -> Option<Seats> {
        match self {
            Self::HeadsUp(seats) => Some(*seats),
            _ => None,
        }
    }

    /// A short label for census tables.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Preflop => "preflop",
            Self::Multiway(_) => "multiway",
            Self::Inverted => "blind-vs-blind",
            Self::HeadsUp(_) => "heads-up",
        }
    }
}

/// A hand reduced to our tree, with the numbers a projection needs.
#[derive(Debug, Clone)]
pub struct Reduction {
    seats: Seats,
    /// Chips in the pot when the flop was dealt.
    pot: Amount,
    /// Chips in that pot contributed by players who had already folded.
    dead: Amount,
    /// Preflop raises made by anyone at the table, capping the depth axis.
    depth: usize,
    /// The last preflop aggressor, when there was one.
    aggressor: Option<usize>,
    /// Smaller of the two stacks behind at the flop.
    effective: Amount,
}

impl Reduction {
    pub fn seats(&self) -> Seats {
        self.seats
    }

    pub fn pot(&self) -> Amount {
        self.pot
    }

    pub fn dead(&self) -> Amount {
        self.dead
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn aggressor(&self) -> Option<usize> {
        self.aggressor
    }

    pub fn effective(&self) -> Amount {
        self.effective
    }

    /// Flop pot in big blinds — the quantity a projected heads-up preflop path
    /// has to match.
    pub fn pot_bb(&self, bblind: Amount) -> f64 {
        f64::from(self.pot) / f64::from(bblind.max(1))
    }

    /// Effective stack in big blinds.
    pub fn effective_bb(&self, bblind: Amount) -> f64 {
        f64::from(self.effective) / f64::from(bblind.max(1))
    }

    /// True when the in-position player took the last preflop aggression.
    pub fn ip_is_aggressor(&self) -> bool {
        self.aggressor == Some(self.seats.ip)
    }

    /// Hand-built reduction, for tests that need one without a whole hand.
    #[cfg(test)]
    pub(crate) fn sample(oop: usize, ip: usize, pot: Amount, depth: usize, aggressor: Option<usize>) -> Self {
        Self {
            seats: Seats { oop, ip },
            pot,
            dead: 0,
            depth,
            aggressor,
            effective: 10_000 - pot / 2,
        }
    }
}

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
                Shape::HeadsUp(Seats { oop: *a, ip: *b })
            }
        }
        many => Shape::Multiway(many.len()),
    }
}

/// Extracts the projection inputs for a hand that reduces to our tree.
///
/// Returns `None` for shapes that do not reduce.
pub fn reduce(record: &Record, outcome: &Outcome) -> Option<Reduction> {
    let seats = shape(record, outcome).seats()?;
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
    Some(Reduction {
        seats,
        pot: flop.pot(),
        dead: flop.pot() - contributed(seats.oop()) - contributed(seats.ip()),
        depth: preflop.iter().filter(|n| matches!(n.step(), Step::BetTo(_, _))).count(),
        aggressor: preflop
            .iter()
            .rev()
            .find(|n| matches!(n.step(), Step::BetTo(_, _)))
            .map(|n| n.seat()),
        effective: record
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEAL: &str = "'d dh p1 TcQc', 'd dh p2 8s4c', 'd dh p3 9c3d', \
                        'd dh p4 Ah4h', 'd dh p5 Th5s', 'd dh p6 6c7s'";

    fn hand(actions: &str) -> (Record, Outcome) {
        let text = format!(
            "variant = 'NT'\nantes = [0, 0, 0, 0, 0, 0]\n\
             blinds_or_straddles = [50, 100, 0, 0, 0, 0]\nmin_bet = 100\n\
             starting_stacks = [10000, 10000, 10000, 10000, 10000, 10000]\n\
             actions = [{DEAL}, {actions}]\n"
        );
        let record = crate::parse(&text, "test").unwrap();
        let outcome = crate::replay(&record).unwrap();
        (record, outcome)
    }

    #[test]
    fn folded_around_is_preflop() {
        let (r, o) = hand("'p3 f', 'p4 f', 'p5 f', 'p6 f', 'p1 f'");
        assert_eq!(shape(&r, &o), Shape::Preflop);
        assert!(reduce(&r, &o).is_none());
    }

    #[test]
    fn blind_versus_blind_is_inverted() {
        let (r, o) = hand("'p3 f', 'p4 f', 'p5 f', 'p6 f', 'p1 cc', 'p2 cc', 'd db 7d5h9d', 'p1 cc', 'p2 cc'");
        assert_eq!(shape(&r, &o), Shape::Inverted);
        assert!(reduce(&r, &o).is_none());
    }

    #[test]
    fn big_blind_versus_a_late_seat_reduces() {
        let (r, o) = hand("'p3 f', 'p4 f', 'p5 f', 'p6 cbr 225', 'p1 f', 'p2 cc', 'd db 7d5h9d', 'p2 cc', 'p6 cc'");
        let seats = shape(&r, &o).seats().expect("reduces");
        assert_eq!(seats.oop(), 1);
        assert_eq!(seats.ip(), 5);
        let reduction = reduce(&r, &o).expect("reduces");
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
        let (r, o) = hand(
            "'p3 f', 'p4 f', 'p5 cbr 225', 'p6 cc', 'p1 f', 'p2 cc', \
             'd db 7d5h9d', 'p2 cc', 'p5 cc', 'p6 cc'",
        );
        assert_eq!(shape(&r, &o), Shape::Multiway(3));
    }
}
