use super::Chips;
use super::N;
use crate::cards::kicks::Kickers;
use crate::cards::ranking::Ranking;
use crate::cards::strength::Strength;
use crate::play::payout::Payout;
use crate::play::seat::State;

// ephemeral data structure that is used to calculate the results of a hand by iterating over hand.actions to calculate side pots, handling every edge case with generalized zero-cost logic
pub struct Showdown {
    payouts: [Payout; N],
    owed: Chips,
    paid: Chips,
    best: Strength,
}

impl From<[Payout; N]> for Showdown {
    fn from(payouts: [Payout; N]) -> Self {
        Self {
            payouts,
            owed: Chips::MIN,
            paid: Chips::MIN,
            best: Strength::from((Ranking::MAX, Kickers::default())),
        }
    }
}

impl Showdown {
    pub fn settlement(mut self) -> [Payout; N] {
        'outer: while let Some(strength) = self.strongest() {
            self.best = strength;
            'inner: while let Some(amount) = self.rewarded() {
                self.owed = amount;
                self.distribute();
                if self.is_complete() {
                    break 'outer;
                } else {
                    continue 'inner;
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
        self.paid = self.owed;
        self.payouts
            .iter()
            .filter(|p| p.strength == self.best)
            .filter(|p| p.risked > self.paid)
            .filter(|p| p.status != State::Folding)
            .map(|p| p.risked)
            .min()
    }
    fn winnings(&self) -> Chips {
        self.payouts
            .iter()
            .map(|p| p.risked)
            .map(|s| std::cmp::min(s, self.owed))
            .map(|s| s.saturating_sub(self.paid))
            .sum()
    }
    fn distribute(&mut self) {
        let chips = self.winnings();
        let mut winners = self
            .payouts
            .iter_mut()
            .filter(|p| p.status != State::Folding)
            .filter(|p| p.strength == self.best)
            .filter(|p| p.risked > self.paid)
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
