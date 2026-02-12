use rbp_core::Chips;
use rbp_cards::*;
use crate::*;

/// Computes chip distributions at showdown.
///
/// Handles all the edge cases of poker settlement: side pots from all-ins,
/// split pots between equal hands, and folded players receiving nothing.
/// The algorithm iterates by strength tier, distributing chips from weakest
/// to strongest hands.
///
/// # Algorithm
///
/// 1. Find the strongest unprocessed hand
/// 2. For that strength tier, compute the side pot they're eligible for
/// 3. Split that pot among all players with that strength
/// 4. Repeat until all chips are distributed
pub struct Showdown {
    payouts: Vec<Settlement>,
    distributing: Chips,
    distributed: Chips,
    best: Strength,
}

impl From<Vec<Settlement>> for Showdown {
    fn from(payouts: Vec<Settlement>) -> Self {
        Self {
            payouts,
            distributing: 0 as Chips,
            distributed: 0 as Chips,
            best: Strength::from((Ranking::MAX, Kickers::default())),
        }
    }
}

impl Showdown {
    /// Distributes all chips and returns final settlements.
    pub fn settle(mut self) -> Vec<Settlement> {
        'winners: while let Some(strength) = self.strongest() {
            self.best = strength;
            'pots: while let Some(amount) = self.remaining() {
                self.distributing = amount;
                self.distribute();
                if self.is_complete() {
                    break 'winners;
                } else {
                    continue 'pots;
                }
            }
        }
        self.payouts
    }
    fn strongest(&self) -> Option<Strength> {
        self.payouts
            .iter()
            .filter(|p| p.strength() < &self.best)
            .filter(|p| p.pnl().status() != State::Folding)
            .map(|p| p.strength())
            .max()
            .cloned()
    }
    fn remaining(&mut self) -> Option<Chips> {
        self.distributed = self.distributing;
        self.payouts
            .iter()
            .filter(|p| p.strength() == &self.best)
            .filter(|p| p.pnl().risked() > self.distributed)
            .filter(|p| p.pnl().status() != State::Folding)
            .map(|p| p.pnl().risked())
            .min()
    }
    fn winnings(&self) -> Chips {
        self.payouts
            .iter()
            .map(|p| p.pnl().risked())
            .map(|s| std::cmp::min(s, self.distributing))
            .map(|s| (s - self.distributed).max(0))
            .sum()
    }
    fn distribute(&mut self) {
        let chips = self.winnings();
        let mut winners = self
            .payouts
            .iter_mut()
            .filter(|p| p.pnl().status() != State::Folding)
            .filter(|p| p.strength() == &self.best)
            .filter(|p| p.pnl().risked() > self.distributed)
            .collect::<Vec<&mut Settlement>>();
        let n = winners.len();
        let share = chips / n as Chips;
        let bonus = chips % n as Chips;
        for winner in winners.iter_mut() {
            winner.add(share);
        }
        for winner in winners.iter_mut().take(bonus as usize) {
            winner.add(1);
        }
    }
    fn is_complete(&self) -> bool {
        let staked = self.payouts.iter().map(|p| p.pnl().risked()).sum::<Chips>();
        let reward = self.payouts.iter().map(|p| p.pnl().reward()).sum::<Chips>();
        staked == reward
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define functions for hand strengths
    fn ace_high() -> Strength {
        Strength::from((Ranking::HighCard(Rank::Ace), Kickers::default()))
    }
    fn one_pair() -> Strength {
        Strength::from((Ranking::OnePair(Rank::Ace), Kickers::default()))
    }
    fn two_pair() -> Strength {
        Strength::from((Ranking::TwoPair(Rank::Ace, Rank::King), Kickers::default()))
    }
    fn triplets() -> Strength {
        Strength::from((Ranking::ThreeOAK(Rank::Ace), Kickers::default()))
    }
    fn the_nuts() -> Strength {
        Strength::from((Ranking::Straight(Rank::Ace), Kickers::default()))
    }

    #[test]
    fn heads_up_showdown() {
        let settlement = Showdown::from(vec![
            Settlement::from((100, State::Betting, ace_high())),
            Settlement::from((100, State::Betting, one_pair())),
        ])
        .settle();
        assert!(settlement[0].pnl().reward() == 0);
        assert!(settlement[1].pnl().reward() == 200);
    }

    #[test]
    fn winners_folded() {
        let settlement = Showdown::from(vec![
            Settlement::from((050, State::Folding, the_nuts())),
            Settlement::from((100, State::Betting, two_pair())),
            Settlement::from((075, State::Folding, the_nuts())),
            Settlement::from((100, State::Betting, one_pair())),
        ])
        .settle();
        assert!(settlement[0].pnl().reward() == 0);
        assert!(settlement[1].pnl().reward() == 325);
        assert!(settlement[2].pnl().reward() == 0);
        assert!(settlement[3].pnl().reward() == 0);
    }

    #[test]
    fn multiway_pot_split() {
        let settlement = Showdown::from(vec![
            Settlement::from((100, State::Betting, two_pair())),
            Settlement::from((100, State::Betting, two_pair())),
            Settlement::from((100, State::Betting, one_pair())),
        ])
        .settle();
        assert!(settlement[0].pnl().reward() == 150);
        assert!(settlement[1].pnl().reward() == 150);
        assert!(settlement[2].pnl().reward() == 0);
    }

    #[test]
    fn multiway_winner_takes_all() {
        let settlement = Showdown::from(vec![
            Settlement::from((200, State::Betting, the_nuts())),
            Settlement::from((150, State::Shoving, triplets())),
            Settlement::from((200, State::Betting, two_pair())),
            Settlement::from((100, State::Shoving, one_pair())),
            Settlement::from((050, State::Folding, the_nuts())),
        ])
        .settle();
        assert!(settlement[0].pnl().reward() == 700);
        assert!(settlement[1].pnl().reward() == 0);
        assert!(settlement[2].pnl().reward() == 0);
        assert!(settlement[3].pnl().reward() == 0);
        assert!(settlement[4].pnl().reward() == 0);
    }

    #[test]
    fn multiway_all_in_with_uneven_stacks() {
        let settlement = Showdown::from(vec![
            Settlement::from((150, State::Shoving, the_nuts())),
            Settlement::from((200, State::Shoving, triplets())),
            Settlement::from((350, State::Shoving, one_pair())),
            Settlement::from((050, State::Shoving, ace_high())),
        ])
        .settle();
        assert!(settlement[0].pnl().reward() == 500);
        assert!(settlement[1].pnl().reward() == 100);
        assert!(settlement[2].pnl().reward() == 150);
        assert!(settlement[3].pnl().reward() == 0);
    }

    #[test]
    fn multiway_all_in_with_side_pot() {
        let settlement = Showdown::from(vec![
            Settlement::from((050, State::Shoving, the_nuts())),
            Settlement::from((100, State::Shoving, triplets())),
            Settlement::from((150, State::Betting, one_pair())),
            Settlement::from((150, State::Betting, ace_high())),
        ])
        .settle();
        assert!(settlement[0].pnl().reward() == 200);
        assert!(settlement[1].pnl().reward() == 150);
        assert!(settlement[2].pnl().reward() == 100);
        assert!(settlement[3].pnl().reward() == 0);
    }

    #[test]
    fn singular_all_in_with_side_pot() {
        let settlement = Showdown::from(vec![
            Settlement::from((050, State::Shoving, two_pair())),
            Settlement::from((100, State::Betting, one_pair())),
            Settlement::from((100, State::Betting, ace_high())),
        ])
        .settle();
        assert!(settlement[0].pnl().reward() == 150);
        assert!(settlement[1].pnl().reward() == 100);
        assert!(settlement[2].pnl().reward() == 0);
    }

    #[test]
    fn singular_all_in_with_side_pot_split() {
        let settlement = Showdown::from(vec![
            Settlement::from((050, State::Shoving, the_nuts())),
            Settlement::from((100, State::Betting, two_pair())),
            Settlement::from((100, State::Betting, two_pair())),
        ])
        .settle();
        assert!(settlement[0].pnl().reward() == 150);
        assert!(settlement[1].pnl().reward() == 50);
        assert!(settlement[2].pnl().reward() == 50);
    }

    #[test]
    fn last_man_standing() {
        let settlement = Showdown::from(vec![
            Settlement::from((050, State::Folding, the_nuts())),
            Settlement::from((100, State::Betting, ace_high())),
            Settlement::from((075, State::Folding, the_nuts())),
            Settlement::from((025, State::Folding, the_nuts())),
        ])
        .settle();
        assert!(settlement[0].pnl().reward() == 0);
        assert!(settlement[1].pnl().reward() == 250);
        assert!(settlement[2].pnl().reward() == 0);
        assert!(settlement[3].pnl().reward() == 0);
    }
}
