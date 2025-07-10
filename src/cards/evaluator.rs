use super::hand::Hand;
use super::kicks::Kickers;
use super::rank::Rank;
use super::ranking::Ranking;
use super::suit::Suit;

#[cfg(not(feature = "shortdeck"))]
const WHEEL: u16 = 0b_1000000001111;
#[cfg(not(feature = "shortdeck"))]
const LOWEST_STRAIGHT_RANK: Rank = Rank::Five;

#[cfg(feature = "shortdeck")]
const WHEEL: u16 = 0b_1000011110000;
#[cfg(feature = "shortdeck")]
const LOWEST_STRAIGHT_RANK: Rank = Rank::Nine;

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
        None.or_else(|| self.find_straight_flush())
            .or_else(|| self.find_4_oak())
            .or_else(|| self.find_3_oak_2_oak())
            .or_else(|| self.find_flush())
            .or_else(|| self.find_straight())
            .or_else(|| self.find_3_oak())
            .or_else(|| self.find_2_oak_2_oak())
            .or_else(|| self.find_2_oak())
            .or_else(|| self.find_1_oak())
            .expect("at least one card in Hand")
    }
    pub fn find_kickers(&self, value: Ranking) -> Kickers {
        match value.n_kickers() {
            0 => Kickers::from(0),
            n => {
                let hand = u16::from(self.0);
                let mask = value.mask();
                let mut rank = hand & mask;
                while n < rank.count_ones() as usize {
                    let last = rank.trailing_zeros();
                    let flip = 1 << last;
                    let skip = !flip;
                    rank &= skip;
                }
                Kickers::from(rank)
            }
        }
    }

    ///

    fn find_1_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(1).map(Ranking::HighCard)
    }
    fn find_2_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(2).map(Ranking::OnePair) // unreachable
    }
    fn find_3_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(3).map(Ranking::ThreeOAK)
    }
    fn find_4_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(4).map(Ranking::FourOAK)
    }
    fn find_2_oak_2_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(2).and_then(|hi| {
            self.find_rank_of_n_oak_skip(2, Some(hi))
                .map(|lo| Ranking::TwoPair(hi, lo))
                .or_else(|| Some(Ranking::OnePair(hi))) // this makes OnePair unreachable
        })
    }
    fn find_3_oak_2_oak(&self) -> Option<Ranking> {
        self.find_rank_of_n_oak(3).and_then(|triple| {
            self.find_rank_of_n_oak_skip(2, Some(triple))
                .map(|paired| Ranking::FullHouse(triple, paired))
        })
    }
    fn find_straight(&self) -> Option<Ranking> {
        self.find_rank_of_straight(self.0).map(Ranking::Straight)
    }
    fn find_flush(&self) -> Option<Ranking> {
        self.find_suit_of_flush().map(|suit| {
            let bits = u16::from(self.0.of(&suit));
            let rank = Rank::from(bits);
            Ranking::Flush(rank)
        })
    }
    fn find_straight_flush(&self) -> Option<Ranking> {
        self.find_suit_of_flush().and_then(|suit| {
            self.find_rank_of_straight_flush(suit)
                .map(Ranking::StraightFlush)
        })
    }

    fn find_rank_of_straight(&self, hand: Hand) -> Option<Rank> {
        let wheel = WHEEL;
        let ranks = u16::from(hand);
        let mut bits = ranks;
        bits &= bits << 1;
        bits &= bits << 1;
        bits &= bits << 1;
        bits &= bits << 1;
        if bits > 0 {
            Some(Rank::from(bits))
        } else if wheel == (wheel & ranks) {
            Some(LOWEST_STRAIGHT_RANK)
        } else {
            None
        }
    }
    fn find_rank_of_straight_flush(&self, suit: Suit) -> Option<Rank> {
        let hand = self.0.of(&suit);
        self.find_rank_of_straight(hand)
    }
    fn find_suit_of_flush(&self) -> Option<Suit> {
        Suit::all()
            .map(|s| u64::from(s))
            .map(|u| u64::from(self.0) & u)
            .map(|n| n.count_ones() as u8)
            .iter()
            .position(|&n| n >= 5)
            .map(|i| Suit::from(i as u8))
    }
    fn find_rank_of_n_oak(&self, n: usize) -> Option<Rank> {
        self.find_rank_of_n_oak_skip(n, None)
    }
    fn find_rank_of_n_oak_skip(&self, n: usize, skip: Option<Rank>) -> Option<Rank> {
        let mut high = u64::from(Rank::Ace) << 4;
        while high > 0 {
            high >>= 4;
            if let Some(skip) = skip {
                let skip = u64::from(skip);
                let skip = high & skip;
                let skip = skip != 0;
                if skip {
                    continue;
                }
            }
            let mine = u64::from(self.0);
            let mine = high & mine;
            let mine = mine.count_ones() >= n as u32;
            if mine {
                return Some(Rank::lo(high));
            }
        }
        None
    }
}

#[cfg(test)]
#[cfg(not(feature = "shortdeck"))]
mod tests {
    use super::*;
    use crate::cards::hand::Hand;

    #[rustfmt::skip]
    #[test]
    fn high_card() {
        let eval = Evaluator::from(Hand::try_from("As Kh Qd Jc 9s").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::HighCard(Rank::Ace));
        assert_eq!(kickers, Kickers::from(vec![Rank::King, Rank::Queen, Rank::Jack, Rank::Nine]));
    }

    #[rustfmt::skip]
    #[test]
    fn one_pair() {
        let eval = Evaluator::from(Hand::try_from("As Ah Kd Qc Js").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::OnePair(Rank::Ace));
        assert_eq!(kickers, Kickers::from(vec![Rank::King, Rank::Queen, Rank::Jack]));
    }

    #[test]
    fn two_pair() {
        let eval = Evaluator::from(Hand::try_from("As Ah Kd Kc Qs").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::TwoPair(Rank::Ace, Rank::King));
        assert_eq!(kickers, Kickers::from(vec![Rank::Queen]));
    }

    #[test]
    fn three_oak() {
        let eval = Evaluator::from(Hand::try_from("As Ah Ad Kc Qs").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::ThreeOAK(Rank::Ace));
        assert_eq!(kickers, Kickers::from(vec![Rank::King, Rank::Queen]));
    }

    #[test]
    fn straight() {
        let eval = Evaluator::from(Hand::try_from("Ts Jh Qd Kc As").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::Straight(Rank::Ace));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn flush() {
        let eval = Evaluator::from(Hand::try_from("As Ks Qs Js 9s").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::Flush(Rank::Ace));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn full_house() {
        let eval = Evaluator::from(Hand::try_from("2s 2h 2d 3c 3s").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::FullHouse(Rank::Two, Rank::Three));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn four_oak() {
        let eval = Evaluator::from(Hand::try_from("As Ah Ad Ac Ks").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::FourOAK(Rank::Ace));
        assert_eq!(kickers, Kickers::from(vec![Rank::King]));
    }

    #[test]
    fn straight_flush() {
        let eval = Evaluator::from(Hand::try_from("Ts Js Qs Ks As").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::StraightFlush(Rank::Ace));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn wheel_straight() {
        let eval = Evaluator::from(Hand::try_from("As 2h 3d 4c 5s").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::Straight(Rank::Five));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn wheel_straight_flush() {
        let eval = Evaluator::from(Hand::try_from("As 2s 3s 4s 5s").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::StraightFlush(Rank::Five));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn seven_card_hand() {
        let eval = Evaluator::from(Hand::try_from("As Ah Kd Kc Qs Jh 9d").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::TwoPair(Rank::Ace, Rank::King));
        assert_eq!(kickers, Kickers::from(vec![Rank::Queen]));
    }

    #[test]
    fn flush_over_straight() {
        let eval = Evaluator::from(Hand::try_from("4h 6h 7h 8h 9h Ts").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::Flush(Rank::Nine));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn full_house_over_flush() {
        let eval = Evaluator::from(Hand::try_from("Kh Ah Ad As Ks Qs Js 9s").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::FullHouse(Rank::Ace, Rank::King));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn four_oak_over_full_house() {
        let eval = Evaluator::from(Hand::try_from("As Ah Ad Ac Ks Kh Qd").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::FourOAK(Rank::Ace));
        assert_eq!(kickers, Kickers::from(vec![Rank::King]));
    }

    #[test]
    fn straight_flush_over_four_oak() {
        let eval = Evaluator::from(Hand::try_from("Ts Js Qs Ks As Ah Ad Ac").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::StraightFlush(Rank::Ace));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn low_straight() {
        let eval = Evaluator::from(Hand::try_from("As 2s 3h 4d 5c 6s").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::Straight(Rank::Six));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn three_pair() {
        let eval = Evaluator::from(Hand::try_from("As Ah Kd Kc Qs Qh Jd").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::TwoPair(Rank::Ace, Rank::King));
        assert_eq!(kickers, Kickers::from(vec![Rank::Queen]));
    }

    #[test]
    fn two_three_oak() {
        let eval = Evaluator::from(Hand::try_from("As Ah Ad Kc Ks Kh Qd").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::FullHouse(Rank::Ace, Rank::King));
        assert_eq!(kickers, Kickers::from(vec![]));
    }
}

#[cfg(test)]
#[cfg(feature = "shortdeck")]
mod tests {
    use super::*;
    use crate::cards::hand::Hand;
    #[test]
    fn shortdeck_wheel_straight() {
        let eval = Evaluator::from(Hand::try_from("6s 7h 8d 9c As").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::Straight(Rank::Nine));
        assert_eq!(kickers, Kickers::from(vec![]));
    }

    #[test]
    fn wheel_straight_flush() {
        let eval = Evaluator::from(Hand::try_from("As 6s 7s 8s 9s").unwrap());
        let ranking = eval.find_ranking();
        let kickers = eval.find_kickers(ranking);
        assert_eq!(ranking, Ranking::StraightFlush(Rank::Nine));
        assert_eq!(kickers, Kickers::from(vec![]));
    }
}
