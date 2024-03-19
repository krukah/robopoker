// ephemeral data structure that is used to calculate the results of a hand by iterating over hand.actions to calculate side pots, handling every edge case with generalized zero-cost logic
pub struct ShowdownMachine {
    payouts: Vec<Payout>,
    next_stake: u32,
    prev_stake: u32,
    next_rank: FullStrength,
}

impl ShowdownMachine {
    pub fn settle(payouts: Vec<Payout>) -> Vec<Payout> {
        let mut this = Self::new(payouts);
        loop {
            this.next_rank();
            loop {
                this.next_stake();
                this.distribute();
                if this.is_complete() {
                    return this.payouts;
                }
            }
        }
    }

    fn new(payouts: Vec<Payout>) -> Self {
        let next_stake = u32::MIN;
        let prev_stake = u32::MIN;
        let next_rank = FullStrength(Strength::MAX, Kickers(Vec::new()));
        Self {
            payouts,
            next_stake,
            prev_stake,
            next_rank,
        }
    }

    fn is_complete(&self) -> bool {
        let staked = self.payouts.iter().map(|p| p.risked).sum::<u32>();
        let reward = self.payouts.iter().map(|p| p.reward).sum::<u32>();
        staked == reward
    }

    fn winnings(&self) -> u32 {
        self.payouts
            .iter()
            .map(|p| p.risked)
            .map(|s| std::cmp::min(s, self.next_stake))
            .map(|s| s.saturating_sub(self.prev_stake))
            .sum()
    }

    fn winners(&mut self) -> Vec<&mut Payout> {
        self.payouts
            .iter_mut()
            .filter(|p| p.strength == self.next_rank)
            .filter(|p| p.risked > self.prev_stake)
            .filter(|p| p.status != BetStatus::Folded)
            .collect()
    }

    fn distribute(&mut self) {
        let winnings = self.winnings();
        let mut winners = self.winners();
        let share = winnings / winners.len() as u32;
        for winner in winners.iter_mut() {
            winner.reward += share;
        }
        let remainder = winnings as usize % winners.len();
        for winner in winners.iter_mut().take(remainder) {
            winner.reward += 1;
        }
    }

    fn next_stake(&mut self) {
        self.prev_stake = self.next_stake;
        self.next_stake = self
            .payouts
            .iter()
            .filter(|p| p.strength == self.next_rank)
            .filter(|p| p.risked > self.prev_stake)
            .filter(|p| p.status != BetStatus::Folded)
            .map(|p| p.risked)
            .min()
            .unwrap();
    }

    fn next_rank(&mut self) {
        self.next_rank = self
            .payouts
            .iter()
            .filter(|p| p.strength < self.next_rank)
            .filter(|p| p.status != BetStatus::Folded)
            .map(|p| p.strength.clone()) //? can we copy, rather than clone, the kickers
            .max()
            .unwrap();
    }
}

use super::{payout::Payout, seat::BetStatus};
use crate::evaluation::strength::{FullStrength, Kickers, Strength};
