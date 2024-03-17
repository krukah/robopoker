#[allow(dead_code)]
type Count = u8;
#[allow(dead_code)]
type RankBits = u16;

struct Evaluation {
    kind_counts: [Count; 5],  // how many i of a kinds are in the hand (redundant)
    suit_counts: [Count; 4],  // how many i suits are in the hand
    rank_counts: [Count; 13], // how many i ranks are in the hand
    suit_bits: [RankBits; 4], // which ranks in which suits are in the hand
    rank_bits: RankBits,      // which ranks are in the hand
}

impl Evaluation {
    pub fn new(cards: &[Card]) -> Self {
        Evaluation {
            kind_counts: Self::kind_counts(cards),
            suit_counts: Self::suit_counts(cards),
            rank_counts: Self::rank_counts(cards),
            suit_bits: Self::suit_bits(cards),
            rank_bits: Self::rank_bits(cards),
        }
    }
    pub fn score(&self) -> u32 {
        todo!()
    }
    pub fn evaluate(&self) -> HandRank {
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

    // HandRank search
    fn find_flush(&self) -> Option<HandRank> {
        let suit = Self::find_suit_of_flush(&self.suit_bits);
        match suit {
            None => None,
            Some(suit) => {
                let rank_bits = self.suit_bits[u8::from(suit) as usize];
                match Self::find_rank_of_straight(rank_bits) {
                    Some(rank) => Some(HandRank::StraightFlush(Rank::from(rank))),
                    None => {
                        let rank = Self::keep_n_msb(rank_bits, 5);
                        Some(HandRank::Flush(Rank::from(rank)))
                    }
                }
            }
        }
    }
    fn find_straight(&self) -> Option<HandRank> {
        todo!()
    }
    fn find_4_oak(&self) -> Option<HandRank> {
        todo!()
    }
    fn find_3_oak_2_oak(&self) -> Option<HandRank> {
        todo!()
    }
    fn find_3_oak(&self) -> Option<HandRank> {
        todo!()
    }
    fn find_2_oak_2_oak(&self) -> Option<HandRank> {
        todo!()
    }
    fn find_2_oak(&self) -> Option<HandRank> {
        todo!()
    }
    fn find_1_oak(&self) -> Option<HandRank> {
        todo!()
    }

    // identifies unique ranks represented in the hand
    fn rank_bits(cards: &[Card]) -> RankBits {
        cards
            .iter()
            .map(|c| c.rank())
            .map(|r| Count::from(r))
            .fold(0u16, |acc, r| acc | 1 << r)
    }
    // identifies unique ranks represented in each suit in the hand
    fn suit_bits(cards: &[Card]) -> [RankBits; 4] {
        let mut suit_bits = [0; 4];
        cards
            .iter()
            .map(|c| [c.suit() as usize, c.rank() as usize])
            .for_each(|[s, r]| suit_bits[s] |= 1 << r);
        suit_bits
    }
    // calculates the number of occurrence cards of each rank in the hand
    fn rank_counts(cards: &[Card]) -> [Count; 13] {
        let mut rank_counts = [0; 13];
        cards
            .iter()
            .map(|c| c.rank())
            .map(|r| Count::from(r) as usize)
            .for_each(|i| rank_counts[i] += 1);
        rank_counts
    }
    // calculates the number of occurences of each suit in the hand
    fn suit_counts(cards: &[Card]) -> [Count; 4] {
        let mut suit_counts = [0; 4];
        cards
            .iter()
            .map(|c| c.suit())
            .map(|s| Count::from(s) as usize)
            .for_each(|i| suit_counts[i] += 1);
        suit_counts
    }
    // calculates the number of occurrences of the number of cards of each rank (i.e. stores 1 at location N if we find N-of-a-kind)
    fn kind_counts(cards: &[Card]) -> [Count; 5] {
        let mut of_a_kind = [0; 5];
        Self::rank_counts(cards)
            .iter()
            .map(|&n| n as usize)
            .for_each(|c| of_a_kind[c] += 1);
        of_a_kind
    }

    fn keep_n_msb(bits: RankBits, n: usize) -> RankBits {
        bits & (1 << n) - 1
    }
    fn keep_n_lsb(bits: RankBits, n: usize) -> RankBits {
        bits & !(!0 << n)
    }
    fn find_rank_of_straight(rank_bits: RankBits) -> Option<Rank> {
        todo!()
    }
    fn find_suit_of_flush(suit_bits: &[RankBits; 4]) -> Option<Suit> {
        let suit = suit_bits.iter().position(|bits| bits.count_ones() >= 5);
        match suit {
            None => None,
            Some(s) => Some(Suit::from(s as u8)),
        }
    }
}

use super::hand_rank::HandRank;
use crate::cards::card::Card;
use crate::cards::rank::Rank;
use crate::cards::suit::Suit;

trait Evaluator {
    fn evaluate(&self) -> HandRank;
    fn score(&self) -> u32;
}
