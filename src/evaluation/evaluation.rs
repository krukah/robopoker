/// we can evaluate a vector of cards lazily by chaining find_* hand rank methods,
/// or we can use ~500MB of memory to store a table of all uniquely evaluated hands.
/// this is a strong tradeoff between space and time complexity.
/// i'll maybe implement LookupEvaluator later
/// Represents an evaluator for evaluating the strength of a hand.

pub trait Evaluator {
    /// Evaluates the strength of the hand.
    ///
    /// # Arguments
    ///
    /// * `cards` - A vector of references to `Card` objects representing the hand.
    ///
    /// # Returns
    ///
    /// The `Strength` of the hand, not including kickers.
    fn evaluate(cards: Vec<&Card>) -> BestHand;

    /// Evaluates the strength of the hand along with the kickers.
    ///
    /// # Arguments
    ///
    /// * `cards` - A vector of references to `Card` objects representing the hand.
    ///
    /// # Returns
    ///
    /// The `FullStrength` of the hand, including the kickers.
    fn evaluate_with_kickers(cards: Vec<&Card>) -> Strength;
}

/// Represents the lazy evaluation of a hand in poker.
/// It stores various sets and counts related to the hand's cards.
pub struct LazyEvaluator {
    hand_set: u32,         // which ranks are in the hand
    suit_set: [u32; 4],    // which ranks are in suits are in the hand
    rank_counts: [u8; 13], // how many i ranks are in the hand. neglect suit
    suit_counts: [u8; 4],  // how many i suits are in the hand. neglect rank
}

impl Evaluator for LazyEvaluator {
    /// Evaluates the strength of the best possible n-card hand, n <= 5.
    fn evaluate(cards: Vec<&Card>) -> BestHand {
        let this = Self::new(&cards);
        let best_hand = this.find_best_hand();
        best_hand
    }
    /// Evaluates the strength of the best possible n-card hand, n <= 5, along with the kickers.
    fn evaluate_with_kickers(cards: Vec<&Card>) -> Strength {
        let this = Self::new(&cards);
        let best_hand = this.find_best_hand();
        let kickers = this.find_kickers(best_hand);
        Strength::new(best_hand, kickers)
    }
}

impl LazyEvaluator {
    /// Creates a new `LazyEval` instance based on the given cards.
    ///
    /// # Arguments
    ///
    /// * `cards` - A vector of references to `Card` objects representing the hand's cards.
    ///
    /// # Returns
    ///
    /// A new `LazyEval` instance.
    fn new(cards: &Vec<&Card>) -> Self {
        Self {
            hand_set: Self::hand_u32(cards),
            suit_set: Self::suit_u32(cards),
            rank_counts: Self::rank_counts(cards),
            suit_counts: Self::suit_counts(cards),
        }
    }

    fn find_best_hand(&self) -> BestHand {
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

    /// Searches for a (straight) flush in the hand.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Strength` of the flush if found, or `None` if not found.
    fn find_flush(&self) -> Option<BestHand> {
        self.find_suit_of_flush().and_then(|suit| {
            self.find_rank_of_straight_flush(suit)
                .map(BestHand::StraightFlush)
                .or_else(|| Some(BestHand::Flush(Rank::from(self.suit_set[suit as usize]))))
        })
    }

    /// Searches for a straight in the hand.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Strength` of the straight if found, or `None` if not found.
    fn find_straight(&self) -> Option<BestHand> {
        self.find_rank_of_straight(self.hand_set)
            .map(|rank| BestHand::Straight(rank))
    }

    /// Searches for a full house (3 of a kind and a pair) in the hand.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Strength` of the full house if found, or `None` if not found.
    fn find_3_oak_2_oak(&self) -> Option<BestHand> {
        self.find_rank_of_n_oak(3).and_then(|triple| {
            self.find_rank_of_n_oak_below(2, triple as usize)
                .map(|couple| BestHand::FullHouse(triple, couple))
        })
    }

    /// Searches for two pairs in the hand.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Strength` of the two pairs if found, or `None` if not found.
    fn find_2_oak_2_oak(&self) -> Option<BestHand> {
        self.find_rank_of_n_oak(2).and_then(|high| {
            self.find_rank_of_n_oak_below(2, high as usize)
                .map(|next| BestHand::TwoPair(high, next))
                .or_else(|| Some(BestHand::OnePair(high)))
        })
    }

    /// Searches for four of a kind in the hand.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Strength` of the four of a kind if found, or `None` if not found.
    fn find_4_oak(&self) -> Option<BestHand> {
        self.find_rank_of_n_oak(4)
            .map(|rank| BestHand::FourOAK(rank))
    }

    /// Searches for three of a kind in the hand.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Strength` of the three of a kind if found, or `None` if not found.
    fn find_3_oak(&self) -> Option<BestHand> {
        self.find_rank_of_n_oak(3)
            .map(|rank| BestHand::ThreeOAK(rank))
    }

    /// Searches for a pair in the hand.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Strength` of the pair if found, or `None` if not found.
    fn find_2_oak(&self) -> Option<BestHand> {
        self.find_rank_of_n_oak(2)
            .map(|rank| BestHand::OnePair(rank))
        // lowkey unreachable because TwoPair short circuits
    }

    /// Searches for a high card in the hand.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Strength` of the high card if found, or `None` if not found.
    fn find_1_oak(&self) -> Option<BestHand> {
        self.find_rank_of_n_oak(1)
            .map(|rank| BestHand::HighCard(rank))
    }

    /// Searches for the suit of a flush in the hand.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Suit` of the flush if found, or `None` if not found.
    fn find_suit_of_flush(&self) -> Option<Suit> {
        self.suit_counts
            .iter()
            .position(|&n| n >= 5)
            .map(|i| Suit::from(i as u8))
    }

    /// Searches for the rank of a straight flush in the hand.
    ///
    /// # Arguments
    ///
    /// * `suit` - The `Suit` of the flush.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Rank` of the straight flush if found, or `None` if not found.
    fn find_rank_of_straight_flush(&self, suit: Suit) -> Option<Rank> {
        let flush_u32 = self.suit_set[suit as usize];
        self.find_rank_of_straight(flush_u32)
    }

    /// Searches for the rank of a straight in the hand.
    ///
    /// # Arguments
    ///
    /// * `hand_u32` - The hand represented as a `u32` bitmask.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Rank` of the straight if found, or `None` if not found.
    fn find_rank_of_straight(&self, hand_u32: u32) -> Option<Rank> {
        let mut mask = hand_u32;
        mask &= mask << 1;
        mask &= mask << 1;
        mask &= mask << 1;
        mask &= mask << 1;
        if mask.count_ones() > 0 {
            return Some(Rank::from(mask));
        } else if Self::wheel() == (Self::wheel() & hand_u32) {
            return Some(Rank::Five);
        } else {
            return None;
        }
    }

    /// Searches for the rank of a specific number of a kind in the hand.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of cards of the same rank to search for.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Rank` of the specified number of a kind if found, or `None` if not found.
    fn find_rank_of_n_oak(&self, n: u8) -> Option<Rank> {
        self.find_rank_of_n_oak_below(n, 13)
    }

    /// Searches for the rank of the next pair in the hand.
    ///
    /// # Arguments
    ///
    /// * `high` - The `Rank` of the highest pair found so far.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Rank` of the next pair if found, or `None` if not found.
    fn find_rank_of_n_oak_below(&self, n: u8, high: usize) -> Option<Rank> {
        self.rank_counts
            .iter()
            .take(high)
            .rev()
            .position(|&r| r >= n)
            .map(|i| high - i - 1)
            .map(|r| Rank::from(r as u8))
    }

    fn find_kickers(&self, strength: BestHand) -> Kickers {
        let n = match strength {
            BestHand::HighCard(_) => 4,
            BestHand::OnePair(_) => 3,
            BestHand::ThreeOAK(_) => 2,
            BestHand::FourOAK(_) => 1,
            BestHand::TwoPair(_, _) => 1,
            _ => return Kickers(vec![]),
        };
        Kickers(
            self.rank_counts
                .iter()
                .enumerate()
                .rev()
                .filter(|(_, x)| **x > 0)
                .filter(|(r, _)| *r != strength.primary() as usize)
                .map(|(i, _)| Rank::from(i as u8))
                .take(n)
                .collect::<Vec<Rank>>(),
        )
    }

    /// Counts the occurrences of each rank in the given cards.
    ///
    /// # Arguments
    ///
    /// * `cards` - A vector of references to `Card` objects representing the hand's cards.
    ///
    /// # Returns
    ///
    /// An array of 13 elements representing the count of each rank.
    fn rank_counts(cards: &Vec<&Card>) -> [u8; 13] {
        let mut rank_counts = [0; 13];
        cards
            .iter()
            .map(|c| c.rank())
            .map(|r| r as usize)
            .for_each(|r| rank_counts[r] += 1);
        rank_counts
    }

    /// Counts the occurrences of each suit in the given cards.
    ///
    /// # Arguments
    ///
    /// * `cards` - A vector of references to `Card` objects representing the hand's cards.
    ///
    /// # Returns
    ///
    /// An array of 4 elements representing the count of each suit.
    fn suit_counts(cards: &Vec<&Card>) -> [u8; 4] {
        let mut suit_counts = [0; 4];
        cards
            .iter()
            .map(|c| c.suit())
            .map(|s| s as usize)
            .for_each(|s| suit_counts[s] += 1);
        suit_counts
    }

    /// Converts the given cards to a `u32` bitmask representing the hand.
    ///
    /// # Arguments
    ///
    /// * `cards` - A vector of references to `Card` objects representing the hand's cards.
    ///
    /// # Returns
    ///
    /// A `u32` bitmask representing the hand.
    fn hand_u32(cards: &Vec<&Card>) -> u32 {
        let mut hand_u32 = 0;
        cards
            .iter()
            .map(|c| c.rank())
            .map(|r| u32::from(r))
            .for_each(|r| hand_u32 |= r);
        hand_u32
    }

    /// Converts the given cards to an array of `u32` bitmasks representing the suits.
    ///
    /// # Arguments
    ///
    /// * `cards` - A vector of references to `Card` objects representing the hand's cards.
    ///
    /// # Returns
    ///
    /// An array of 4 `u32` bitmasks representing the suits.
    fn suit_u32(cards: &Vec<&Card>) -> [u32; 4] {
        let mut suit_u32 = [0; 4];
        cards
            .iter()
            .map(|c| (c.suit(), c.rank()))
            .map(|(s, r)| (s as usize, u32::from(r)))
            .for_each(|(s, r)| suit_u32[s] |= r);
        suit_u32
    }

    fn wheel() -> u32 {
        0b00000000000000000001000000001111
    }
}

use std::vec;

use super::strength::{BestHand, Kickers, Strength};
use crate::cards::card::Card;
use crate::cards::rank::Rank;
use crate::cards::suit::Suit;
