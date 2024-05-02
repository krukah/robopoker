// ephemeral data structure that is used to calculate the results of a hand by iterating over hand.actions to calculate side pots, handling every edge case with generalized zero-cost logic
pub struct Showdown {
    payouts: Vec<Payout>,
    next_stake: u32,
    prev_stake: u32,
    next_strength: Strength,
}

impl Showdown {
    pub fn concede(mut payouts: Vec<Payout>) -> Vec<Payout> {
        let reward = payouts.iter().map(|p| p.risked).sum::<u32>();
        let winner = payouts
            .iter_mut()
            .find(|p| p.status != BetStatus::Folded)
            .unwrap();
        winner.reward = reward;
        payouts
    }
    pub fn settle(payouts: Vec<Payout>) -> Vec<Payout> {
        let mut this = Self::new(payouts);
        'strength: while let Some(strength) = this.next_strength() {
            this.next_strength = strength;
            'stake: while let Some(stake) = this.next_stake() {
                this.next_stake = stake;
                this.distribute();
                if this.is_complete() {
                    break 'strength;
                } else {
                    continue 'stake;
                }
            }
        }
        this.payouts
    }

    fn new(payouts: Vec<Payout>) -> Self {
        let next_rank = Strength::new(BestHand::MAX, Kickers(Vec::new()));
        let next_stake = u32::MIN;
        let prev_stake = u32::MIN;
        Self {
            payouts,
            next_strength: next_rank,
            next_stake,
            prev_stake,
        }
    }

    fn is_complete(&self) -> bool {
        let staked = self.payouts.iter().map(|p| p.risked).sum::<u32>();
        let reward = self.payouts.iter().map(|p| p.reward).sum::<u32>();
        staked == reward
    }

    fn next_strength(&mut self) -> Option<Strength> {
        self.payouts
            .iter()
            .filter(|p| p.strength < self.next_strength)
            .filter(|p| p.status != BetStatus::Folded)
            .map(|p| p.strength.clone()) //? can we copy, rather than clone, the kickers
            .max()
    }

    fn next_stake(&mut self) -> Option<u32> {
        self.prev_stake = self.next_stake;
        self.payouts
            .iter()
            .filter(|p| p.strength == self.next_strength)
            .filter(|p| p.risked > self.prev_stake)
            .filter(|p| p.status != BetStatus::Folded)
            .map(|p| p.risked)
            .min()
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
            .filter(|p| p.strength == self.next_strength)
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
}

use crate::evaluation::strength::{BestHand, Kickers, Strength};
use crate::gameplay::{payout::Payout, seat::BetStatus};
