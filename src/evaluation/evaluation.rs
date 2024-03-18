/// we can evaluate a vector of cards lazily by chaining find_* hand rank methods,
/// or we can use ~500MB of memory to store a table of all uniquely evaluated hands.
/// this is a strong tradeoff between space and time complexity.
/// i'll maybe implement LookupEvaluator later
trait Evaluator {
    fn evaluate(&self) -> Strength;
}

pub struct LazyEvaluator {
    hand_u32: u32,         // which ranks are in the hand
    suit_u32: [u32; 4],    // which ranks in which suits are in the hand
    rank_counts: [u8; 13], // how many i ranks are in the hand. neglect suit
    suit_counts: [u8; 4],  // how many i suits are in the hand. neglect rank
}

impl LazyEvaluator {
    pub fn new(cards: &Vec<&Card>) -> Self {
        LazyEvaluator {
            hand_u32: Self::hand_u32(cards),
            suit_u32: Self::suit_u32(cards),
            rank_counts: Self::rank_counts(cards),
            suit_counts: Self::suit_counts(cards),
        }
    }
    pub fn evaluate(&self) -> Strength {
        self.find_flush()
            .or_else(|| self.find_4_oak())
            .or_else(|| self.find_3_oak_2_oak())
            .or_else(|| self.find_straight())
            .or_else(|| self.find_3_oak())
            .or_else(|| self.find_2_oak_2_oak())
            .or_else(|| self.find_2_oak())
            .or_else(|| self.find_1_oak())
            .unwrap()
    }

    // searches for HandRank
    fn find_flush(&self) -> Option<Strength> {
        self.find_suit_of_flush().and_then(|suit| {
            self.find_rank_of_straight_flush(suit)
                .map(Strength::StraightFlush)
                .or_else(|| Some(Strength::Flush(Rank::from(self.suit_u32[suit as usize]))))
        })
    }
    fn find_straight(&self) -> Option<Strength> {
        self.find_rank_of_straight(self.hand_u32)
            .map(|rank| Strength::Straight(rank))
    }
    fn find_3_oak_2_oak(&self) -> Option<Strength> {
        self.find_rank_of_n_oak(3).and_then(|triple| {
            self.find_rank_of_next_pair(triple)
                .map(|couple| Strength::FullHouse(triple, couple))
        })
    }
    fn find_2_oak_2_oak(&self) -> Option<Strength> {
        self.find_rank_of_n_oak(2).and_then(|high| {
            self.find_rank_of_next_pair(high)
                .map(|next| Strength::TwoPair(high, next))
                .or_else(|| Some(Strength::OnePair(high)))
        })
    }
    fn find_4_oak(&self) -> Option<Strength> {
        self.find_rank_of_n_oak(4)
            .map(|rank| Strength::FourOAK(rank))
    }
    fn find_3_oak(&self) -> Option<Strength> {
        self.find_rank_of_n_oak(3)
            .map(|rank| Strength::ThreeOAK(rank))
    }
    fn find_2_oak(&self) -> Option<Strength> {
        self.find_rank_of_n_oak(2)
            .map(|rank| Strength::OnePair(rank))
        // lowkey unreachable because TwoPair short circuits
    }
    fn find_1_oak(&self) -> Option<Strength> {
        self.find_rank_of_n_oak(1)
            .map(|rank| Strength::HighCard(rank))
    }

    // sub-searches for Rank and Suit
    fn find_suit_of_flush(&self) -> Option<Suit> {
        self.suit_counts
            .iter()
            .position(|&n| n >= 5)
            .map(|i| Suit::from(i as u8))
    }
    fn find_rank_of_straight_flush(&self, suit: Suit) -> Option<Rank> {
        let flush_u32 = self.suit_u32[suit as usize];
        self.find_rank_of_straight(flush_u32)
    }
    fn find_rank_of_straight(&self, hand_u32: u32) -> Option<Rank> {
        let mut mask = hand_u32;
        mask &= mask << 1;
        mask &= mask << 1;
        mask &= mask << 1;
        mask &= mask << 1;
        if mask.count_ones() > 0 {
            return Some(Rank::from(mask));
        } else if Rank::wheel() == (Rank::wheel() & hand_u32) {
            return Some(Rank::Five);
        } else {
            return None;
        }
    }
    fn find_rank_of_n_oak(&self, /* high=13 */ n: u8) -> Option<Rank> {
        self.rank_counts
            .iter()
            .rev()
            .position(|&r| r >= n)
            .map(|i| 13 - i - 1)
            .map(|r| Rank::from(r as u8))
    }
    fn find_rank_of_next_pair(&self, /* n=2 */ high: Rank) -> Option<Rank> {
        self.rank_counts
            .iter()
            .take(high as usize)
            .rev()
            .position(|&r| r >= 2)
            .map(|i| high as usize - i - 1)
            .map(|r| Rank::from(r as u8))
    }

    // sub-constructors for LazyEvaluator
    fn rank_counts(cards: &Vec<&Card>) -> [u8; 13] {
        let mut rank_counts = [0; 13];
        cards
            .iter()
            .map(|c| c.rank())
            .map(|r| r as usize)
            .for_each(|r| rank_counts[r] += 1);
        rank_counts
    }
    fn suit_counts(cards: &Vec<&Card>) -> [u8; 4] {
        let mut suit_counts = [0; 4];
        cards
            .iter()
            .map(|c| c.suit())
            .map(|s| s as usize)
            .for_each(|s| suit_counts[s] += 1);
        suit_counts
    }
    fn hand_u32(cards: &Vec<&Card>) -> u32 {
        let mut hand_u32 = 0;
        cards
            .iter()
            .map(|c| c.rank())
            .map(|r| u32::from(r))
            .for_each(|r| hand_u32 |= r);
        hand_u32
    }
    fn suit_u32(cards: &Vec<&Card>) -> [u32; 4] {
        let mut suit_u32 = [0; 4];
        cards
            .iter()
            .map(|c| (c.suit(), c.rank()))
            .map(|(s, r)| (s as usize, u32::from(r)))
            .for_each(|(s, r)| suit_u32[s] |= r);
        suit_u32
    }
}

use super::strength::Strength;
use crate::cards::card::Card;
use crate::cards::rank::Rank;
use crate::cards::suit::Suit;
