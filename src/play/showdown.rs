use super::game::Game;
use super::Chips;
use crate::cards::kicks::Kickers;
use crate::cards::ranking::Ranking;
use crate::cards::strength::Strength;
use crate::play::{payout::Payout, seat::Status};

// ephemeral data structure that is used to calculate the results of a hand by iterating over hand.actions to calculate side pots, handling every edge case with generalized zero-cost logic
pub struct Showdown {
    payouts: Vec<Payout>,
    next_stake: Chips,
    prev_stake: Chips,
    next_strength: Strength, // make option to handle initial state
}

impl From<Game> for Showdown {
    fn from(game: Game) -> Self {
        todo!()
    }
}

impl Showdown {
    pub fn concede(mut payouts: Vec<Payout>) -> Vec<Payout> {
        let reward = payouts.iter().map(|p| p.risked).sum::<Chips>();
        let winner = payouts
            .iter_mut()
            .find(|p| p.status != Status::Folding)
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
        let next_rank = Strength::from((Ranking::MAX, Kickers::from(0u16)));
        let next_stake = Chips::MIN;
        let prev_stake = Chips::MIN;
        Self {
            payouts,
            next_strength: next_rank,
            next_stake,
            prev_stake,
        }
    }

    fn is_complete(&self) -> bool {
        let staked = self.payouts.iter().map(|p| p.risked).sum::<Chips>();
        let reward = self.payouts.iter().map(|p| p.reward).sum::<Chips>();
        staked == reward
    }

    fn next_strength(&mut self) -> Option<Strength> {
        self.payouts
            .iter()
            .filter(|p| p.strength < self.next_strength)
            .filter(|p| p.status != Status::Folding)
            .map(|p| p.strength)
            .max()
    }

    fn next_stake(&mut self) -> Option<Chips> {
        self.prev_stake = self.next_stake;
        self.payouts
            .iter()
            .filter(|p| p.strength == self.next_strength)
            .filter(|p| p.risked > self.prev_stake)
            .filter(|p| p.status != Status::Folding)
            .map(|p| p.risked)
            .min()
    }

    fn winnings(&self) -> Chips {
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
            .filter(|p| p.status != Status::Folding)
            .collect()
    }

    fn distribute(&mut self) {
        let winnings = self.winnings();
        let mut winners = self.winners();
        let share = winnings / winners.len() as Chips;
        for winner in winners.iter_mut() {
            winner.reward += share;
        }
        let remainder = winnings as usize % winners.len();
        for winner in winners.iter_mut().take(remainder) {
            winner.reward += 1;
        }
    }
}
