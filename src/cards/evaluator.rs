use super::card::Card;
use super::hand::Hand;
use super::kicks::Kickers;
use super::rank::Rank;
use super::ranking::Ranking;
use super::suit::Suit;

/// A lazy evaluator for a hand's strength.
///
/// Using a compact representation of the Hand, we search for
/// the highest Value hand using bitwise operations. I should
/// benchmark this and compare to a massive HashMap<Hand, Value> lookup implementation.
pub struct Evaluator(Hand);
impl From<Hand> for Evaluator {
    fn from(h: Hand) -> Self {
        Self(h)
    }
}

impl Evaluator {
    pub fn find_ranking(&self) -> Ranking {
        None.or_else(|| self.find_flush())
            .or_else(|| self.find_4_oak())
            .or_else(|| self.find_3_oak_2_oak())
            .or_else(|| self.find_straight())
            .or_else(|| self.find_3_oak())
            .or_else(|| self.find_2_oak_2_oak())
            .or_else(|| self.find_2_oak())
            .or_else(|| self.find_1_oak())
            .expect("at least one card in Hand")
    }
    pub fn find_kickers(&self, value: Ranking) -> Kickers {
        let n = match value {
            Ranking::FourOAK(_) | Ranking::TwoPair(_, _) => 1,
            Ranking::HighCard(_) => 4,
            Ranking::OnePair(_) => 3,
            Ranking::ThreeOAK(_) => 2,
            _ => return Kickers::from(0u16),
        };
        let mask = match value {
            Ranking::TwoPair(hi, lo) => u16::from(hi) | u16::from(lo),
            Ranking::HighCard(hi)
            | Ranking::OnePair(hi)
            | Ranking::ThreeOAK(hi)
            | Ranking::FourOAK(hi) => u16::from(hi),
            _ => unreachable!(),
        };
        let mut bits = mask & self.rank_masks();
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
            self.find_rank_of_n_oak_under(2, Some(hi))
                .map(|lo| Ranking::TwoPair(hi, lo))
                .or_else(|| Some(Ranking::OnePair(hi)))
        })
    }
    fn find_3_oak_2_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(3).and_then(|trips| {
            self.find_rank_of_n_oak_under(2, Some(trips))
                .map(|pairs| Ranking::FullHouse(trips, pairs))
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

    fn find_rank_of_straight(&self, hand: u16) -> Option<Rank> {
        const WHEEL: u16 = 0b_1000000001111;
        let mut bits = hand;
        bits &= bits << 1;
        bits &= bits << 1;
        bits &= bits << 1;
        bits &= bits << 1;
        if bits > 0 {
            Some(Rank::from(bits))
        } else if WHEEL == (WHEEL & hand) {
            Some(Rank::Five)
        } else {
            None
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
    fn find_rank_of_n_oak_under(&self, oak: usize, rank: Option<Rank>) -> Option<Rank> {
        let rank = rank.map(|c| u8::from(c)).unwrap_or(13) as u64;
        let mask = (1u64 << (4 * rank)) - 1;
        let hand = u64::from(self.0) & mask;
        let mut mask = 0b_1111_u64 << (4 * (rank)) >> 4;
        while mask > 0 {
            if oak <= (hand & mask).count_ones() as usize {
                let rank = mask.trailing_zeros() / 4;
                let rank = Rank::from(rank as u8);
                return Some(rank);
            }
            mask >>= 4;
        }
        None
    }
    fn find_rank_of_n_oak(&self, n: usize) -> Option<Rank> {
        self.find_rank_of_n_oak_under(n, None)
    }
    ///

    /// rank_masks:
    /// Masks,
    /// which ranks are in the hand, neglecting suit
    fn rank_masks(&self) -> u16 {
        Vec::<Card>::from(self.0)
            .iter()
            .map(|c| c.rank())
            .map(|r| u16::from(r))
            .fold(0, |acc, r| acc | r)
    }
    /// suit_count:
    /// [Count; 4],
    /// how many suits (i) are in the hand. neglect rank
    fn suit_count(&self) -> [u8; 4] {
        Vec::<Card>::from(self.0)
            .iter()
            .map(|c| c.suit())
            .map(|s| u8::from(s))
            .fold([0; 4], |mut counts, s| {
                counts[s as usize] += 1;
                counts
            })
    }
    /// suit_masks:
    /// [Masks; 4],
    /// which ranks are in the hand, grouped by suit
    /// TODO: this could accept Suit as argument
    /// and return the u16 mask for that suit directly
    fn suit_masks(&self) -> [u16; 4] {
        Vec::<Card>::from(self.0)
            .iter()
            .map(|c| (c.suit(), c.rank()))
            .map(|(s, r)| (u8::from(s), u16::from(r)))
            .fold([0; 4], |mut suits, (s, r)| {
                suits[s as usize] |= r;
                suits
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::card::Card;
    use crate::cards::rank::Rank;
    use crate::cards::suit::Suit;

    fn evaluate_hand(cards: Vec<(Rank, Suit)>) -> Ranking {
        let hand = Hand::from(
            cards
                .into_iter()
                .map(|(r, s)| Card::from((r, s)))
                .collect::<Vec<Card>>(),
        );
        let evaluator = Evaluator::from(hand);
        evaluator.find_ranking()
    }

    #[test]
    fn high_card() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::King, Suit::Heart),
            (Rank::Queen, Suit::Diamond),
            (Rank::Jack, Suit::Club),
            (Rank::Nine, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::HighCard(Rank::Ace));
    }

    #[test]
    fn one_pair() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::King, Suit::Diamond),
            (Rank::Queen, Suit::Club),
            (Rank::Jack, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::OnePair(Rank::Ace));
    }

    #[test]
    fn two_pair() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::King, Suit::Diamond),
            (Rank::King, Suit::Club),
            (Rank::Queen, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::TwoPair(Rank::Ace, Rank::King));
    }

    #[test]
    fn three_oak() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::Ace, Suit::Diamond),
            (Rank::King, Suit::Club),
            (Rank::Queen, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::ThreeOAK(Rank::Ace));
    }

    #[test]
    fn straight() {
        let hand = vec![
            (Rank::Ten, Suit::Spade),
            (Rank::Jack, Suit::Heart),
            (Rank::Queen, Suit::Diamond),
            (Rank::King, Suit::Club),
            (Rank::Ace, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::Straight(Rank::Ace));
    }

    #[test]
    fn flush() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::King, Suit::Spade),
            (Rank::Queen, Suit::Spade),
            (Rank::Jack, Suit::Spade),
            (Rank::Nine, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::Flush(Rank::Ace));
    }

    #[test]
    fn full_house() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::Ace, Suit::Diamond),
            (Rank::King, Suit::Club),
            (Rank::King, Suit::Spade),
        ];
        assert_eq!(
            evaluate_hand(hand),
            Ranking::FullHouse(Rank::Ace, Rank::King)
        );
    }

    #[test]
    fn four_oak() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::Ace, Suit::Diamond),
            (Rank::Ace, Suit::Club),
            (Rank::King, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::FourOAK(Rank::Ace));
    }

    #[test]
    fn straight_flush() {
        let hand = vec![
            (Rank::Ten, Suit::Spade),
            (Rank::Jack, Suit::Spade),
            (Rank::Queen, Suit::Spade),
            (Rank::King, Suit::Spade),
            (Rank::Ace, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::StraightFlush(Rank::Ace));
    }

    #[test]
    fn wheel_straight() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Two, Suit::Heart),
            (Rank::Three, Suit::Diamond),
            (Rank::Four, Suit::Club),
            (Rank::Five, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::Straight(Rank::Five));
    }

    #[test]
    fn wheel_straight_flush() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Two, Suit::Spade),
            (Rank::Three, Suit::Spade),
            (Rank::Four, Suit::Spade),
            (Rank::Five, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::StraightFlush(Rank::Five));
    }

    #[test]
    fn seven_card_hand() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::King, Suit::Diamond),
            (Rank::King, Suit::Club),
            (Rank::Queen, Suit::Spade),
            (Rank::Jack, Suit::Heart),
            (Rank::Nine, Suit::Diamond),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::TwoPair(Rank::Ace, Rank::King));
    }

    #[test]
    fn flush_vs_straight() {
        let hand = vec![
            (Rank::Four, Suit::Heart),
            (Rank::Six, Suit::Heart),
            (Rank::Seven, Suit::Heart),
            (Rank::Eight, Suit::Heart),
            (Rank::Nine, Suit::Heart),
            (Rank::Ten, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::Flush(Rank::Nine));
    }

    #[test]
    fn full_house_vs_flush() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::Ace, Suit::Diamond),
            (Rank::King, Suit::Spade),
            (Rank::King, Suit::Heart),
            (Rank::Queen, Suit::Spade),
            (Rank::Jack, Suit::Spade),
        ];
        assert_eq!(
            evaluate_hand(hand),
            Ranking::FullHouse(Rank::Ace, Rank::King)
        );
    }

    #[test]
    fn two_three_oak() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::Ace, Suit::Diamond),
            (Rank::King, Suit::Club),
            (Rank::King, Suit::Spade),
            (Rank::King, Suit::Heart),
            (Rank::Queen, Suit::Diamond),
        ];
        assert_eq!(
            evaluate_hand(hand),
            Ranking::FullHouse(Rank::Ace, Rank::King)
        );
    }

    #[test]
    fn four_oak_vs_full_house() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::Ace, Suit::Diamond),
            (Rank::Ace, Suit::Club),
            (Rank::King, Suit::Spade),
            (Rank::King, Suit::Heart),
            (Rank::Queen, Suit::Diamond),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::FourOAK(Rank::Ace));
    }

    #[test]
    fn straight_flush_vs_four_oak() {
        let hand = vec![
            (Rank::Ten, Suit::Spade),
            (Rank::Jack, Suit::Spade),
            (Rank::Queen, Suit::Spade),
            (Rank::King, Suit::Spade),
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::Ace, Suit::Diamond),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::StraightFlush(Rank::Ace));
    }

    #[test]
    fn low_straight() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Two, Suit::Spade),
            (Rank::Three, Suit::Heart),
            (Rank::Four, Suit::Diamond),
            (Rank::Five, Suit::Club),
            (Rank::Six, Suit::Spade),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::Straight(Rank::Six));
    }

    #[test]
    fn three_pair() {
        let hand = vec![
            (Rank::Ace, Suit::Spade),
            (Rank::Ace, Suit::Heart),
            (Rank::King, Suit::Diamond),
            (Rank::King, Suit::Club),
            (Rank::Queen, Suit::Spade),
            (Rank::Queen, Suit::Heart),
            (Rank::Jack, Suit::Diamond),
        ];
        assert_eq!(evaluate_hand(hand), Ranking::TwoPair(Rank::Ace, Rank::King));
    }
}
