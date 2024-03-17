pub struct Evaluation {
    hand_u32: u32,              // which ranks are in the hand
    hand_u32_by_suit: [u32; 4], // which ranks in which suits are in the hand
    suit_counts: [u8; 4],       // how many i suits are in the hand. neglect rank
    rank_counts: [u8; 13],      // how many i ranks are in the hand. neglect suit
}

impl Evaluation {
    pub fn new(cards: Vec<&Card>) -> Self {
        Evaluation {
            hand_u32: Self::hand_u32(cards.clone()),
            hand_u32_by_suit: Self::hand_u32_by_suit(cards.clone()),
            rank_counts: Self::rank_counts(cards.clone()),
            suit_counts: Self::suit_counts(cards.clone()),
        }
    }
    pub fn evaluate(&self) -> HandRank {
        self.find_flushes()
            .or_else(|| self.find_4_oak())
            .or_else(|| self.find_3_oak_2_oak())
            .or_else(|| self.find_straight())
            .or_else(|| self.find_3_oak())
            .or_else(|| self.find_2_oak_2_oak())
            .or_else(|| self.find_2_oak())
            .or_else(|| self.find_1_oak())
            .unwrap()
    }
    pub fn score(&self) -> u32 {
        let eval = self.evaluate();
        println!("{:?}", eval);
        match eval {
            HandRank::HighCard(r) => u8::from(r) as u32,
            HandRank::OnePair(r) => 100 + u8::from(r) as u32,
            HandRank::TwoPair(r1, r2) => 200 + (u8::from(r1) + u8::from(r2)) as u32,
            HandRank::ThreeOfAKind(r) => 300 + u8::from(r) as u32,
            HandRank::Straight(r) => 400 + u8::from(r) as u32,
            HandRank::Flush(r) => 500 + u8::from(r) as u32,
            HandRank::FullHouse(r1, r2) => 600 + (u8::from(r1) + u8::from(r2)) as u32,
            HandRank::FourOfAKind(r) => 700 + u8::from(r) as u32,
            HandRank::StraightFlush(r) => 800 + u8::from(r) as u32,
        }
    }

    // HandRank search
    fn find_flushes(&self) -> Option<HandRank> {
        match self.find_flush_suit() {
            None => None,
            Some(suit) => match self.find_straight_flush_high(suit) {
                None => {
                    let flush_u32 = self.hand_u32_by_suit[suit as usize];
                    let rank = CardRank::from(flush_u32);
                    Some(HandRank::Flush(rank))
                }
                Some(rank) => Some(HandRank::StraightFlush(rank)),
            },
        }
    }
    fn find_straight(&self) -> Option<HandRank> {
        match self.find_straight_high(self.hand_u32) {
            None => None,
            Some(r) => Some(HandRank::Straight(r)),
        }
    }
    fn find_4_oak(&self) -> Option<HandRank> {
        match self.find_n_oak_high(4) {
            None => None,
            Some(rank) => Some(HandRank::FourOfAKind(CardRank::from(rank as u8))),
        }
    }
    fn find_3_oak_2_oak(&self) -> Option<HandRank> {
        match self.find_n_oak_high(3) {
            None => None,
            Some(triplet) => match self
                .rank_counts
                .iter()
                .take(triplet as usize) // remove the triplet rank
                .rev()
                .position(|&n| n >= 2) // find a pair
                .map(|i| triplet as usize - i - 1)
            {
                Some(pair) => Some(HandRank::FullHouse(
                    CardRank::from(triplet as u8),
                    CardRank::from(pair as u8),
                )),
                None => None,
            },
        }
    }
    fn find_3_oak(&self) -> Option<HandRank> {
        match self.find_n_oak_high(3) {
            None => None,
            Some(r) => Some(HandRank::ThreeOfAKind(CardRank::from(r as u8))),
        }
    }
    fn find_2_oak_2_oak(&self) -> Option<HandRank> {
        match self.find_n_oak_high(2) {
            None => None,
            Some(high_pair) => match self
                .rank_counts
                .iter()
                .take(high_pair as usize)
                .rev()
                .position(|&n| n >= 2)
                .map(|i| high_pair as usize - i - 1)
            {
                None => Some(HandRank::OnePair(high_pair)),
                Some(next_pair) => Some(HandRank::TwoPair(
                    CardRank::from(high_pair as u8),
                    CardRank::from(next_pair as u8),
                )),
            },
        }
    }
    fn find_2_oak(&self) -> Option<HandRank> {
        match self.find_n_oak_high(2) {
            None => None,
            Some(r) => Some(HandRank::OnePair(CardRank::from(r as u8))),
        }
    }
    fn find_1_oak(&self) -> Option<HandRank> {
        match self.find_n_oak_high(1) {
            None => None,
            Some(r) => Some(HandRank::HighCard(CardRank::from(r as u8))),
        }
    }

    // sub-HandRank search
    fn find_n_oak_high(&self, n: u8) -> Option<CardRank> {
        match self
            .rank_counts
            .iter()
            .rev()
            .position(|&r| r >= n)
            .map(|i| 13 - i - 1)
        {
            Some(rank) => Some(CardRank::from(rank as u8)),
            None => None,
        }
    }
    fn find_straight_flush_high(&self, suit: Suit) -> Option<CardRank> {
        let flush = self.hand_u32_by_suit[suit as usize];
        self.find_straight_high(flush)
    }
    fn find_straight_high(&self, hand_u32: u32) -> Option<CardRank> {
        let rank_u32 = (0..5).fold(hand_u32, |acc, i| acc & hand_u32 << i);
        if rank_u32.count_ones() > 0 {
            return Some(CardRank::from(rank_u32));
        }
        let five_u32 = 0b00000000000000000001000000001111;
        if (hand_u32 & five_u32) == five_u32 {
            return Some(CardRank::Five);
        }
        None
        // xxxxxxxxxxxxxxxxxxxx11111.......
        // xxxxxxxxxxxxxxxxxxx11111........
        // xxxxxxxxxxxxxxxxxx11111.........
        // xxxxxxxxxxxxxxxxx11111..........
        // xxxxxxxxxxxxxxxx11111...........
        // --------------------------------
        // xxxxxxxxxxxxxxxx00001...........
    }
    fn find_flush_suit(&self) -> Option<Suit> {
        match self.suit_counts.iter().position(|&n| n >= 5) {
            None => None,
            Some(suit) => Some(Suit::from(suit as u8)),
        }
    }

    // constructors
    // identifies unique ranks represented in the hand
    fn hand_u32(cards: Vec<&Card>) -> u32 {
        let mut union = 0;
        cards
            .iter()
            .map(|c| u32::from(c.rank()))
            .for_each(|u| union |= u);
        union
    }
    // identifies unique ranks represented in each suit in the hand
    fn hand_u32_by_suit(cards: Vec<&Card>) -> [u32; 4] {
        let mut suit_bits = [0; 4];
        cards
            .iter()
            .map(|c| [c.suit() as usize, c.rank() as usize])
            .for_each(|[s, r]| suit_bits[s] |= 1 << r);
        suit_bits
    }
    // calculates the number of occurrence cards of each rank in the hand
    fn rank_counts(cards: Vec<&Card>) -> [u8; 13] {
        let mut rank_counts = [0; 13];
        cards
            .iter()
            .map(|c| c.rank())
            .map(|r| u8::from(r) as usize)
            .for_each(|r| rank_counts[r] += 1);
        rank_counts
    }
    // calculates the number of occurences of each suit in the hand
    fn suit_counts(cards: Vec<&Card>) -> [u8; 4] {
        let mut suit_counts = [0; 4];
        cards
            .iter()
            .map(|c| c.suit())
            .map(|s| u8::from(s) as usize)
            .for_each(|s| suit_counts[s] += 1);
        suit_counts
    }
}

use super::hand_rank::HandRank;
use crate::cards::card::Card;
use crate::cards::rank::Rank as CardRank;
use crate::cards::suit::Suit;

trait Evaluator {
    fn evaluate(&self) -> HandRank;
    fn score(&self) -> u32;
}
