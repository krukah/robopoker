use super::Chips;
use crate::cards::kicks::Kickers;
use crate::cards::ranking::Ranking;
use crate::cards::strength::Strength;
use crate::play::payout::Payout;
use crate::play::seat::State;

// ephemeral data structure that is used to calculate the results of a hand by iterating over hand.actions to calculate side pots, handling every edge case with generalized zero-cost logic
pub struct Showdown {
    payouts: Vec<Payout>,
    distributing: Chips,
    distributed: Chips,
    best: Strength,
}

impl From<Vec<Payout>> for Showdown {
    fn from(payouts: Vec<Payout>) -> Self {
        Self {
            payouts,
            distributing: Chips::MIN,
            distributed: Chips::MIN,
            best: Strength::from((Ranking::MAX, Kickers::default())),
        }
    }
}

impl Showdown {
    pub fn settle(mut self) -> Vec<Payout> {
        'winners: while let Some(strength) = self.strongest() {
            self.best = strength;
            'winnings: while let Some(amount) = self.rewarded() {
                self.distributing = amount;
                self.distribute();
                if self.is_complete() {
                    break 'winners;
                } else {
                    continue 'winnings;
                }
            }
        }
        self.payouts
    }
    fn strongest(&self) -> Option<Strength> {
        self.payouts
            .iter()
            .filter(|p| p.strength < self.best)
            .filter(|p| p.status != State::Folding)
            .map(|p| p.strength)
            .max()
    }
    fn rewarded(&mut self) -> Option<Chips> {
        self.distributed = self.distributing;
        self.payouts
            .iter()
            .filter(|p| p.strength == self.best)
            .filter(|p| p.risked > self.distributed)
            .filter(|p| p.status != State::Folding)
            .map(|p| p.risked)
            .min()
    }
    fn winnings(&self) -> Chips {
        self.payouts
            .iter()
            .map(|p| p.risked)
            .map(|s| std::cmp::min(s, self.distributing))
            .map(|s| s.saturating_sub(self.distributed))
            .sum()
    }
    fn distribute(&mut self) {
        let chips = self.winnings();
        let mut winners = self
            .payouts
            .iter_mut()
            .filter(|p| p.status != State::Folding)
            .filter(|p| p.strength == self.best)
            .filter(|p| p.risked > self.distributed)
            .collect::<Vec<&mut Payout>>();
        let share = chips / winners.len() as Chips;
        let bonus = chips as usize % winners.len();
        for winner in winners.iter_mut() {
            winner.reward += share;
        }
        for winner in winners.iter_mut().take(bonus) {
            winner.reward += 1;
        }
    }
    fn is_complete(&self) -> bool {
        let staked = self.payouts.iter().map(|p| p.risked).sum::<Chips>();
        let reward = self.payouts.iter().map(|p| p.reward).sum::<Chips>();
        staked == reward
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::rank::Rank;
    // Define functions for hand strengths
    fn six_high() -> Strength {
        Strength::from((Ranking::HighCard(Rank::Ace), Kickers::default()))
    }
    fn one_pair() -> Strength {
        Strength::from((Ranking::OnePair(Rank::Ace), Kickers::default()))
    }
    fn two_pair() -> Strength {
        Strength::from((Ranking::TwoPair(Rank::Ace, Rank::King), Kickers::default()))
    }
    fn triplets() -> Strength {
        Strength::from((Ranking::ThreeOAK(Rank::Ace), Kickers::default())) // CORRECT
    }
    fn the_nuts() -> Strength {
        Strength::from((Ranking::Straight(Rank::Ace), Kickers::default()))
    }

    #[test]
    fn heads_up_showdown() {
        let settlement = Showdown::from(vec![
            Payout::from((100, State::Playing, six_high())),
            Payout::from((100, State::Playing, one_pair())),
        ])
        .settle();
        assert!(settlement[0].reward == 0);
        assert!(settlement[1].reward == 200);
    }

    #[test]
    fn winners_folded() {
        let settlement = Showdown::from(vec![
            Payout::from((050, State::Folding, the_nuts())),
            Payout::from((100, State::Playing, two_pair())),
            Payout::from((075, State::Folding, the_nuts())),
            Payout::from((100, State::Playing, one_pair())),
        ])
        .settle();
        assert!(settlement[0].reward == 0);
        assert!(settlement[1].reward == 325);
        assert!(settlement[2].reward == 0);
        assert!(settlement[3].reward == 0);
    }

    #[test]
    fn multiway_pot_split() {
        let settlement = Showdown::from(vec![
            Payout::from((100, State::Playing, two_pair())),
            Payout::from((100, State::Playing, two_pair())),
            Payout::from((100, State::Playing, one_pair())),
        ])
        .settle();
        assert!(settlement[0].reward == 150);
        assert!(settlement[1].reward == 150);
        assert!(settlement[2].reward == 0);
    }

    #[test]
    fn multiway_winner_takes_all() {
        let settlement = Showdown::from(vec![
            Payout::from((200, State::Playing, the_nuts())),
            Payout::from((150, State::Shoving, triplets())),
            Payout::from((200, State::Playing, two_pair())),
            Payout::from((100, State::Shoving, one_pair())),
            Payout::from((050, State::Folding, the_nuts())),
        ])
        .settle();
        assert!(settlement[0].reward == 700);
        assert!(settlement[1].reward == 0);
        assert!(settlement[2].reward == 0);
        assert!(settlement[3].reward == 0);
        assert!(settlement[4].reward == 0);
    }

    #[test]
    fn multiway_all_in_with_uneven_stacks() {
        let settlement = Showdown::from(vec![
            Payout::from((150, State::Shoving, the_nuts())),
            Payout::from((200, State::Shoving, triplets())),
            Payout::from((350, State::Shoving, one_pair())),
            Payout::from((050, State::Shoving, six_high())),
        ])
        .settle();
        assert!(settlement[0].reward == 500);
        assert!(settlement[1].reward == 100);
        assert!(settlement[2].reward == 150);
        assert!(settlement[3].reward == 0);
    }

    #[test]
    fn multiway_all_in_with_side_pot() {
        let settlement = Showdown::from(vec![
            Payout::from((050, State::Shoving, the_nuts())),
            Payout::from((100, State::Shoving, triplets())),
            Payout::from((150, State::Playing, one_pair())),
            Payout::from((150, State::Playing, six_high())),
        ])
        .settle();
        assert!(settlement[0].reward == 200);
        assert!(settlement[1].reward == 150);
        assert!(settlement[2].reward == 100);
        assert!(settlement[3].reward == 0);
    }

    #[test]
    fn singular_all_in_with_side_pot() {
        let settlement = Showdown::from(vec![
            Payout::from((050, State::Shoving, two_pair())),
            Payout::from((100, State::Playing, one_pair())),
            Payout::from((100, State::Playing, six_high())),
        ])
        .settle();
        assert!(settlement[0].reward == 150);
        assert!(settlement[1].reward == 100);
        assert!(settlement[2].reward == 0);
    }

    #[test]
    fn singular_all_in_with_side_pot_split() {
        let settlement = Showdown::from(vec![
            Payout::from((050, State::Shoving, the_nuts())),
            Payout::from((100, State::Playing, two_pair())),
            Payout::from((100, State::Playing, two_pair())),
        ])
        .settle();
        assert!(settlement[0].reward == 150);
        assert!(settlement[1].reward == 50);
        assert!(settlement[2].reward == 50);
    }

    #[test]
    fn last_man_standing() {
        let settlement = Showdown::from(vec![
            Payout::from((050, State::Folding, the_nuts())),
            Payout::from((100, State::Playing, six_high())),
            Payout::from((075, State::Folding, the_nuts())),
            Payout::from((025, State::Folding, the_nuts())),
        ])
        .settle();
        assert!(settlement[0].reward == 0);
        assert!(settlement[1].reward == 250);
        assert!(settlement[2].reward == 0);
        assert!(settlement[3].reward == 0);
    }
}
