use super::hand::Hand;
use super::observation::Observation;
use super::suit::Suit;
use itertools::Itertools;
use rand::seq::SliceRandom;

/// an array of 4 unique Suits represents
/// any of the 4! = 24 elements in the Suit permutation group.
/// by assuming a "canonical" order of suits (C < D < H < S),
/// we use [Suit; 4] to map C -> P[0], D -> P[1], H -> P[2], S -> P[3].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Permutation([Suit; 4]);

impl Permutation {
    pub const fn identity() -> Self {
        Self(Suit::all())
    }
    pub fn transform(&self, ref observation: Observation) -> Observation {
        Observation::from((
            self.permute(&observation.secret()),
            self.permute(&observation.public()),
        ))
    }
    pub fn permute(&self, hand: &Hand) -> Hand {
        Suit::all()
            .iter()
            .map(|suit| self.suited(hand, suit))
            .fold(Hand::empty(), |acc, x| Hand::add(acc, x))
    }
    pub fn exhaust() -> [Self; 24] {
        Suit::all()
            .into_iter()
            .permutations(4)
            .map(|p| p.try_into().unwrap())
            .map(|p| Self(p))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
    pub fn random() -> Self {
        let ref mut rng = rand::thread_rng();
        let mut suits = Suit::all();
        suits.shuffle(rng);
        Self(suits)
    }
    fn suited(&self, hand: &Hand, suit: &Suit) -> Hand {
        let cards = u64::from(*suit) & u64::from(*hand);
        let old = *suit;
        let new = self.get(suit);
        let shift = new as i8 - old as i8;
        if shift >= 0 {
            Hand::from(cards << shift as u64)
        } else {
            Hand::from(cards >> shift.abs() as u64)
        }
    }
    fn get(&self, suit: &Suit) -> Suit {
        self.0[*suit as usize]
    }
    fn set(&mut self, old: &Suit, new: &Suit) {
        self.0[*old as usize] = *new;
    }
}

/// this yields a (sorta) unique Permutation
/// that will map an Observation to its canonical form.
/// uniqueness only applies to Suits that are present in the Observation.
/// i.e. an Observation with only 2 Suits represented
/// leaves 2 additional degrees of freedom.
impl From<Observation> for Permutation {
    fn from(observation: Observation) -> Self {
        let mut permutation = Self::identity();
        observation
            .into_iter()
            .zip(Suit::all())
            .for_each(|(old, new)| permutation.set(&old, &new));
        permutation
    }
}

impl std::fmt::Display for Permutation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Suit::all()
            .into_iter()
            .inspect(|s| write!(f, "{} -> {}\n", s, self.get(s)).unwrap())
            .count();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::suit::Suit;

    #[test]
    fn map_identity() {
        let identity = Permutation::identity();
        assert!(identity.get(&Suit::C) == Suit::C);
        assert!(identity.get(&Suit::D) == Suit::D);
        assert!(identity.get(&Suit::H) == Suit::H);
        assert!(identity.get(&Suit::S) == Suit::S);
    }

    #[test]
    fn map_arbitrary() {
        let permutation = Permutation([Suit::H, Suit::S, Suit::C, Suit::D]);
        assert!(permutation.get(&Suit::C) == Suit::H);
        assert!(permutation.get(&Suit::D) == Suit::S);
        assert!(permutation.get(&Suit::H) == Suit::C);
        assert!(permutation.get(&Suit::S) == Suit::D);
    }

    #[test]
    fn permute_simple() {
        let permutation = Permutation([Suit::H, Suit::C, Suit::S, Suit::D]);
        let hearts = Hand::from(0b_0100_0100_0100_0100_0100_0100_0100_0100_u64);
        let spades = Hand::from(0b_1000_1000_1000_1000_1000_1000_1000_1000_u64);
        assert!(permutation.permute(&hearts) == spades);
    }

    #[test]
    fn permute_unique() {
        let ref hand = Hand::from("Ac Kd Qh Js");
        let mut unique = std::collections::HashSet::new();
        let n = Permutation::exhaust()
            .into_iter()
            .map(|p| p.permute(hand))
            .inspect(|h| assert!(unique.insert(*h)))
            .count();
        assert!(n == 24);
    }

    #[test]
    fn permute_complex() {
        let permutation = Permutation([Suit::D, Suit::H, Suit::C, Suit::S]);
        let original = Hand::from(0b_1010_1010_1010_1010__0100_0100_0100_0100_u64);
        let permuted = Hand::from(0b_1100_1100_1100_1100__0001_0001_0001_0001_u64);
        assert!(permutation.permute(&original) == permuted);
    }

    #[test]
    fn permute_rotation() {
        let permutation = Permutation([Suit::S, Suit::C, Suit::D, Suit::H]);
        let original = Hand::from("Ac Kd Qh Js");
        let permuted = Hand::from("As Kc Qd Jh");
        assert!(permutation.permute(&original) == permuted);
    }

    #[test]
    fn permute_interior() {
        let permutation = Permutation([Suit::C, Suit::H, Suit::D, Suit::S]);
        let original = Hand::from("2c 3d 4h 5s");
        let permuted = Hand::from("2c 3h 4d 5s");
        assert!(permutation.permute(&original) == permuted);
    }

    #[test]
    fn permute_identity() {
        let permutation = Permutation::identity();
        let hand = Hand::random();
        assert!(permutation.permute(&hand) == hand);
    }
}
