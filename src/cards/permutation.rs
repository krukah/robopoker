use super::hand::Hand;
use super::observation::Observation;
use super::suit::Suit;

/// an array of 4 unique Suits represents
/// any of the 4! = 24 possible permutations.
/// by assuming a "canonical" order of suits (C < D < H < S),
/// we map C -> P[0], D -> P[1], H -> P[2], S -> P[3].
#[derive(PartialEq, Eq)]
pub struct Permutation([Suit; 4]);

impl Permutation {
    pub const fn identity() -> Self {
        Self(Suit::all())
    }
    pub fn permute(&self, hand: &Hand) -> Hand {
        Hand::from(
            Suit::all()
                .iter()
                .map(|suit| self.suited(hand, suit))
                .fold(0u64, |acc, x| acc | x),
        )
    }
    fn suited(&self, hand: &Hand, suit: &Suit) -> u64 {
        let cards = u64::from(*suit) & u64::from(*hand);
        let shift = self.shift(suit);
        if shift >= 0 {
            cards >> (shift as u64)
        } else {
            cards << (shift.abs() as u64)
        }
    }
    fn shift(&self, suit: &Suit) -> i8 {
        let old = *suit;
        let new = self.map(suit);
        new as i8 - old as i8
    }
    fn map(&self, suit: &Suit) -> Suit {
        self.0[*suit as usize]
    }
}

impl From<&Observation> for Permutation {
    fn from(observation: &Observation) -> Self {
        let secret = observation.secret().suit_count();
        let public = observation.public().suit_count();
        let mut suits = Suit::all()
            .into_iter()
            .enumerate()
            .map(|(i, suit)| (secret[i], public[i], suit))
            .collect::<Vec<(u8, u8, Suit)>>();
        suits.sort_by(|a, b| b.cmp(a));
        let permutation = suits
            .into_iter()
            .map(|(_, _, suit)| suit)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Self(permutation)
    }
}

#[cfg(test)]
mod tests {
    use crate::cards::suit::Suit;

    use super::*;

    #[test]
    fn identity_map() {
        let identity = Permutation::identity();
        assert!(identity.map(&Suit::C) == Suit::C);
        assert!(identity.map(&Suit::D) == Suit::D);
        assert!(identity.map(&Suit::H) == Suit::H);
        assert!(identity.map(&Suit::S) == Suit::S);
    }

    #[test]
    fn arbitrary_map() {
        let permutation = Permutation([Suit::H, Suit::S, Suit::C, Suit::D]);
        assert!(permutation.map(&Suit::C) == Suit::H);
        assert!(permutation.map(&Suit::D) == Suit::S);
        assert!(permutation.map(&Suit::H) == Suit::C);
        assert!(permutation.map(&Suit::S) == Suit::D);
    }

    #[test]
    fn identity_transform() {
        let identity = Permutation::identity();
        let hand = Hand::random();
        assert!(identity.permute(&hand) == hand);
    }

    #[test]
    fn simple_transform() {
        let permutation = Permutation([Suit::H, Suit::C, Suit::S, Suit::D]);
        let hearts = Hand::from(0b_0100_0100_0100_0100_0100_0100_0100_0100_u64);
        let spades = Hand::from(0b_1000_1000_1000_1000_1000_1000_1000_1000_u64);
        assert!(permutation.permute(&hearts) == spades);
    }

    #[test]
    fn arbitrary_transform() {
        let permutation = Permutation([Suit::D, Suit::H, Suit::C, Suit::S]);
        let original = Hand::from(0b_1010_1010_1010_1010__0100_0100_0100_0100_u64);
        let permuted = Hand::from(0b_1100_1100_1100_1100__0001_0001_0001_0001_u64);
        assert!(permutation.permute(&original) == permuted);
    }
}
