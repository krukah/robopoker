use super::card::Card;
use super::hand::Hand;
use super::kicks::Kickers;
use super::rank::Rank;
use super::strength::Strength;
use super::suit::Suit;
use super::value::Ranking;

/// A lazy evaluator for a hand's strength.
///
/// Using a compact representation of the Hand, we search for
/// the highest Value hand using bitwise operations. I should
/// benchmark this and compare to a massive HashMap<Hand, Value> lookup implementation.
/// alias types useful for readability here
pub struct Evaluator(Hand);
impl From<Hand> for Evaluator {
    fn from(h: Hand) -> Self {
        Self(h)
    }
}

impl From<Evaluator> for Strength {
    fn from(e: Evaluator) -> Self {
        let value = e.find_ranking();
        let kicks = e.find_kickers(value);
        Self::from((value, kicks))
    }
}

impl Evaluator {
    /// rank_masks:
    /// Masks,
    /// which ranks are in the hand, neglecting suit
    fn rank_masks(&self) -> u32 {
        Vec::<Card>::from(self.0)
            .iter()
            .map(|c| c.rank())
            .map(|r| r as u32)
            .fold(0, |acc, r| acc | r)
    }
    /// rank_count:
    /// [Count; 13],
    /// how many ranks (i) are in the hand. neglect suit
    fn rank_count(&self) -> [u8; 13] {
        Vec::<Card>::from(self.0)
            .iter()
            .map(|c| c.rank())
            .map(|r| r as usize)
            .fold([0; 13], |mut counts, r| {
                counts[r] += 1;
                counts
            })
    }
    /// suit_count:
    /// [Count; 4],
    /// how many suits (i) are in the hand. neglect rank
    fn suit_count(&self) -> [u8; 4] {
        Vec::<Card>::from(self.0)
            .iter()
            .map(|c| c.suit())
            .map(|s| s as usize)
            .fold([0; 4], |mut counts, s| {
                counts[s] += 1;
                counts
            })
    }
    /// suit_masks:
    /// [Masks; 4],
    /// which ranks are in the hand, grouped by suit
    fn suit_masks(&self) -> [u32; 4] {
        Vec::<Card>::from(self.0)
            .iter()
            .map(|c| (c.suit(), c.rank()))
            .map(|(s, r)| (s as usize, u32::from(r)))
            .fold([0; 4], |mut suits, (s, r)| {
                suits[s] |= r;
                suits
            })
    }

    ///

    fn find_ranking(&self) -> Ranking {
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
    fn find_kickers(&self, value: Ranking) -> Kickers {
        let n = match value {
            Ranking::HighCard(_) => 4,
            Ranking::OnePair(_) => 3,
            Ranking::ThreeOAK(_) => 2,
            Ranking::FourOAK(_) | Ranking::TwoPair(_, _) => 1,
            _ => return Kickers::from(0u32),
        };
        let mask = match value {
            Ranking::HighCard(hi)
            | Ranking::OnePair(hi)
            | Ranking::ThreeOAK(hi)
            | Ranking::FourOAK(hi) => !u32::from(hi),
            Ranking::TwoPair(hi, lo) => !u32::from(hi) & !u32::from(lo),
            _ => unreachable!(),
        };
        let mut bits = !mask & self.rank_masks();
        while bits.count_ones() > n {
            bits &= !(1 << bits.trailing_zeros());
        }
        Kickers::from(bits)
    }

    ///

    fn find_1_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(1).map(Ranking::HighCard)
    }
    fn find_2_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(2).map(Ranking::OnePair)
    }
    fn find_3_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(3).map(Ranking::ThreeOAK)
    }
    fn find_4_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(4).map(Ranking::FourOAK)
    }
    fn find_2_oak_2_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(2).and_then(|hi| {
            self.find_rank_of_n_oak_below(2, hi as usize)
                .map(|lo| Ranking::TwoPair(hi, lo))
                .or_else(|| Some(Ranking::OnePair(hi)))
        })
    }
    fn find_3_oak_2_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(3).and_then(|three| {
            self.find_rank_of_n_oak_below(2, three as usize)
                .map(|two| Ranking::FullHouse(three, two))
        })
    }
    fn find_straight(&self) -> Option<Ranking> {
        self.find_rank_of_straight(self.rank_masks())
            .map(Ranking::Straight)
    }
    fn find_flush(&self) -> Option<Ranking> {
        self.find_suit_of_flush().and_then(|suit| {
            self.find_rank_of_straight_flush(suit)
                .map(Ranking::StraightFlush)
                .or_else(|| {
                    let bits = self.suit_masks();
                    let bits = bits[suit as usize];
                    let rank = Rank::from(bits);
                    Some(Ranking::Flush(rank))
                })
        })
    }

    ///

    fn find_rank_of_straight(&self, u32_cards: u32) -> Option<Rank> {
        const WHEEL: u32 = 0b0000000000000000000_1000000001111;
        let mut bits = u32_cards;
        bits &= bits << 1;
        bits &= bits << 1;
        bits &= bits << 1;
        bits &= bits << 1;
        if bits > 0 {
            return Some(Rank::from(bits));
        } else if WHEEL == (WHEEL & u32_cards) {
            return Some(Rank::Five);
        } else {
            return None;
        }
    }
    fn find_rank_of_straight_flush(&self, suit: Suit) -> Option<Rank> {
        let bits = self.suit_masks();
        let bits = bits[suit as usize];
        self.find_rank_of_straight(bits)
    }
    fn find_suit_of_flush(&self) -> Option<Suit> {
        self.suit_count()
            .iter()
            .position(|&n| n >= 5)
            .map(|i| Suit::from(i as u8))
    }
    fn find_rank_of_n_oak_below(&self, n: u8, high: usize) -> Option<Rank> {
        // TODO
        // performance bottleneck
        self.rank_count()
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
