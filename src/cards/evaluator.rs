use super::card::Card;
use super::hand::Hand;
use super::kicks::Kicks;
use super::rank::Rank;
use super::strength::Strength;
use super::suit::Suit;
use super::value::Value;

type Masks = u32; // could be u16
type Count = u8; // could pack this entire struct into a super efficient u128 probably
type Cards<'a> = &'a Vec<Card>; // could be Hand(u64) or generic over Iterator<Card>

/// A lazy evaluator for a hand's strength.
///
/// Using a compact representation of the Hand, we search for
/// the highest Value hand using bitwise operations. I should
/// benchmark this and compare to a massive HashMap<Hand, Value> lookup implementation.
/// alias types useful for readability here
pub struct Evaluator {
    rank_masks: Masks,       // which ranks are in the hand, neglecting suit
    suit_masks: [Masks; 4],  // which ranks are in the hand, grouped by suit
    suit_count: [Count; 4],  // how many suits (i) are in the hand. neglect rank
    rank_count: [Count; 13], // how many ranks (i) are in the hand. neglect suit
}

impl From<Hand> for Evaluator {
    fn from(hand: Hand) -> Self {
        let ref cards = Vec::<Card>::from(hand);
        Self {
            rank_masks: Self::rank_masks(cards),
            suit_masks: Self::suit_masks(cards),
            suit_count: Self::suit_count(cards),
            rank_count: Self::rank_count(cards),
        }
    }
}

impl From<Evaluator> for Strength {
    fn from(evaluator: Evaluator) -> Self {
        let value = evaluator.find_value();
        let kicks = evaluator.find_kicks(value);
        Self::from((value, kicks))
    }
}

impl Evaluator {
    fn rank_count(cards: Cards) -> [u8; 13] {
        cards
            .iter()
            .map(|c| c.rank())
            .map(|r| r as usize)
            .fold([0; 13], |mut counts, r| {
                counts[r] += 1;
                counts
            })
    }
    fn suit_count(cards: Cards) -> [u8; 4] {
        cards
            .iter()
            .map(|c| c.suit())
            .map(|s| s as usize)
            .fold([0; 4], |mut counts, s| {
                counts[s] += 1;
                counts
            })
    }
    fn suit_masks(cards: Cards) -> [u32; 4] {
        cards
            .iter()
            .map(|c| (c.suit(), c.rank()))
            .map(|(s, r)| (s as usize, u32::from(r)))
            .fold([0; 4], |mut suits, (s, r)| {
                suits[s] |= r;
                suits
            })
    }
    fn rank_masks(cards: Cards) -> u32 {
        cards
            .iter()
            .map(|c| c.rank())
            .map(|r| r as u32)
            .fold(0, |acc, r| acc | r)
    }

    ///

    fn find_value(&self) -> Value {
        self.find_flush()
            .or_else(|| self.find_4_oak())
            .or_else(|| self.find_3_oak_2_oak())
            .or_else(|| self.find_straight())
            .or_else(|| self.find_3_oak())
            .or_else(|| self.find_2_oak_2_oak())
            .or_else(|| self.find_2_oak())
            .or_else(|| self.find_1_oak())
            .expect("at least one card in Hand")
    }
    fn find_kicks(&self, value: Value) -> Kicks {
        // remove the value cards from the hand
        // MUST FIX THIS
        // MUST FIX THIS
        Kicks::from(Hand::from(0))
    }

    ///

    fn find_1_oak(&self) -> Option<Value> {
        self.find_rank_of_n_oak(1).map(Value::HighCard)
    }
    fn find_2_oak(&self) -> Option<Value> {
        self.find_rank_of_n_oak(2).map(Value::OnePair)
    }
    fn find_3_oak(&self) -> Option<Value> {
        self.find_rank_of_n_oak(3).map(Value::ThreeOAK)
    }
    fn find_4_oak(&self) -> Option<Value> {
        self.find_rank_of_n_oak(4).map(Value::FourOAK)
    }
    fn find_2_oak_2_oak(&self) -> Option<Value> {
        self.find_rank_of_n_oak(2).and_then(|hi| {
            self.find_rank_of_n_oak_below(2, hi as usize)
                .map(|lo| Value::TwoPair(hi, lo))
                .or_else(|| Some(Value::OnePair(hi)))
        })
    }
    fn find_3_oak_2_oak(&self) -> Option<Value> {
        self.find_rank_of_n_oak(3).and_then(|three| {
            self.find_rank_of_n_oak_below(2, three as usize)
                .map(|two| Value::FullHouse(three, two))
        })
    }
    fn find_straight(&self) -> Option<Value> {
        self.find_rank_of_straight(self.rank_masks)
            .map(Value::Straight)
    }
    fn find_flush(&self) -> Option<Value> {
        self.find_suit_of_flush().and_then(|suit| {
            self.find_rank_of_straight_flush(suit)
                .map(Value::StraightFlush)
                .or_else(|| {
                    let mask = self.suit_masks[suit as usize];
                    let rank = Rank::from(mask);
                    Some(Value::Flush(rank))
                })
        })
    }

    ///

    fn find_rank_of_straight(&self, u32_cards: u32) -> Option<Rank> {
        const WHEEL: u32 = 0b_0000_0000_0000_0000_0001_0000_0000_1111;
        let mut mask = u32_cards;
        mask &= mask << 1;
        mask &= mask << 1;
        mask &= mask << 1;
        mask &= mask << 1;
        if mask > 0 {
            return Some(Rank::from(mask));
        } else if WHEEL == (WHEEL & u32_cards) {
            return Some(Rank::Five);
        } else {
            return None;
        }
    }
    fn find_rank_of_straight_flush(&self, suit: Suit) -> Option<Rank> {
        let flush = self.suit_masks[suit as usize];
        self.find_rank_of_straight(flush)
    }
    fn find_suit_of_flush(&self) -> Option<Suit> {
        self.suit_count
            .iter()
            .position(|&n| n >= 5)
            .map(|i| Suit::from(i as u8))
    }
    fn find_rank_of_n_oak_below(&self, n: u8, high: usize) -> Option<Rank> {
        self.rank_count
            .iter()
            .take(high)
            .rev()
            .position(|&r| r >= n)
            .map(|i| high - i - 1)
            .map(|r| Rank::from(r as u8))
    }
    fn find_rank_of_n_oak(&self, n: u8) -> Option<Rank> {
        self.find_rank_of_n_oak_below(n, 13)
    }
}
