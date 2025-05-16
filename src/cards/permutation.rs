use super::hand::Hand;
use super::observation::Observation;
use super::suit::Suit;
use crate::Arbitrary;

/// an array of 4 unique Suits represents
/// any of the 4! = 24 elements in the Suit permutation group.
/// by assuming a "canonical" order of suits (C < D < H < S),
/// we use [Suit; 4] to map C -> P[0], D -> P[1], H -> P[2], S -> P[3].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Permutation([Suit; 4]);

/// this yields consistent, though, possibly non-unique,
/// Permutation that will map an Observation to its canonical form.
/// suits are sorted co-lexicographically by the number of cards
/// that they are represented by in hole cards and on the board.
/// ties are broken by the arbitrary enum impl Ord for Suit !
impl From<&Observation> for Permutation {
    fn from(observation: &Observation) -> Self {
        let mut permutation = Suit::all();
        let mut colex = Suit::all().map(|suit| Self::colex(&observation, &suit));
        colex.sort_by(Self::order);
        colex
            .into_iter()
            .enumerate()
            .map(|(i, (suit, _, _))| (suit as usize, Suit::from(i as u8)))
            .for_each(|(index, value)| permutation[index] = value);
        Self(permutation)
    }
}

impl Permutation {
    /// the image of an Observation under a Permutation
    /// is computed from applying the Permutation to
    /// its constituent Hands, pocket and public
    pub fn permute(&self, observation: &Observation) -> Observation {
        Observation::from((
            self.image(observation.pocket()),
            self.image(observation.public()),
        ))
    }

    /// the image of a hand under a permutation
    /// is the union of its shifted sub-Hands
    pub fn image(&self, hand: &Hand) -> Hand {
        Suit::all()
            .iter()
            .map(|suit| self.shift(suit, hand))
            .fold(Hand::empty(), |acc, x| Hand::add(acc, x))
    }

    /// impose order by breaking symmetries
    /// 1. who has fewer cards in pocket?
    /// 2. who has fewer cards on public?
    /// 3. who has weaker pocket cards?
    /// 4. who has weaker public cards?
    /// 5. who has stronger pocket cards? redundant, tbh. due to 3.
    /// 6. who has stronger public cards?
    /// 7. tie delegates to Suit order
    fn order(hearts: &(Suit, Hand, Hand), spades: &(Suit, Hand, Hand)) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
            .then_with(|| hearts.1.size().cmp(&spades.1.size()))
            .then_with(|| hearts.2.size().cmp(&spades.2.size()))
            .then_with(|| hearts.1.min_rank().cmp(&spades.1.min_rank()))
            .then_with(|| hearts.2.min_rank().cmp(&spades.2.min_rank()))
            .then_with(|| hearts.1.max_rank().cmp(&spades.1.max_rank()))
            .then_with(|| hearts.2.max_rank().cmp(&spades.2.max_rank()))
            .then_with(|| hearts.0.cmp(&spades.0)) // tiebreaker
    }

    /// there's this thing called co-lexicographic order
    /// which is a total ordering on some sub sets of cards
    /// in our case Observation. it implements Order at different
    /// scopes to break symmetries of strategically identical Observations.
    fn colex(observation: &Observation, suit: &Suit) -> (Suit, Hand, Hand) {
        let pocket = observation.pocket().of(suit);
        let public = observation.public().of(suit);
        (*suit, pocket, public)
    }

    /// the hand here gets filtered by the "old" suit
    /// and then we bitshift so that it is in its "new" suit
    /// e.g. Full Hand -> Hearts Hand -> Spades Hand
    fn shift(&self, suit: &Suit, hand: &Hand) -> Hand {
        let old = *suit;
        let new = self.map(suit);
        let shift = new as i8 - old as i8;
        let cards = u64::from(*suit) & u64::from(*hand);
        if shift >= 0 {
            Hand::from(cards << shift as u64)
        } else {
            Hand::from(cards >> shift.abs() as u64)
        }
    }
    /// get the image of a Suit under a Permutation
    fn map(&self, suit: &Suit) -> Suit {
        self.0[*suit as usize]
    }

    pub const fn identity() -> Self {
        Self(Suit::all())
    }
    pub const fn exhaust() -> [Self; 24] {
        [
            Self([Suit::C, Suit::D, Suit::H, Suit::S]),
            Self([Suit::C, Suit::D, Suit::S, Suit::H]),
            Self([Suit::C, Suit::H, Suit::D, Suit::S]),
            Self([Suit::C, Suit::H, Suit::S, Suit::D]),
            Self([Suit::C, Suit::S, Suit::D, Suit::H]),
            Self([Suit::C, Suit::S, Suit::H, Suit::D]),
            Self([Suit::D, Suit::C, Suit::H, Suit::S]),
            Self([Suit::D, Suit::C, Suit::S, Suit::H]),
            Self([Suit::D, Suit::H, Suit::C, Suit::S]),
            Self([Suit::D, Suit::H, Suit::S, Suit::C]),
            Self([Suit::D, Suit::S, Suit::C, Suit::H]),
            Self([Suit::D, Suit::S, Suit::H, Suit::C]),
            Self([Suit::H, Suit::C, Suit::D, Suit::S]),
            Self([Suit::H, Suit::C, Suit::S, Suit::D]),
            Self([Suit::H, Suit::D, Suit::C, Suit::S]),
            Self([Suit::H, Suit::D, Suit::S, Suit::C]),
            Self([Suit::H, Suit::S, Suit::C, Suit::D]),
            Self([Suit::H, Suit::S, Suit::D, Suit::C]),
            Self([Suit::S, Suit::C, Suit::D, Suit::H]),
            Self([Suit::S, Suit::C, Suit::H, Suit::D]),
            Self([Suit::S, Suit::D, Suit::C, Suit::H]),
            Self([Suit::S, Suit::D, Suit::H, Suit::C]),
            Self([Suit::S, Suit::H, Suit::C, Suit::D]),
            Self([Suit::S, Suit::H, Suit::D, Suit::C]),
        ]
    }
}

impl Arbitrary for Permutation {
    fn random() -> Self {
        use rand::prelude::IndexedRandom;
        let ref mut rng = rand::rng();
        Self::exhaust().choose(rng).copied().unwrap()
    }
}

impl std::fmt::Display for Permutation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Suit::all()
            .iter()
            .map(|s| writeln!(f, "{} -> {}", s, self.map(s)))
            .last()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::suit::Suit;

    #[test]
    fn map_identity() {
        let identity = Permutation::identity();
        assert!(identity.map(&Suit::C) == Suit::C);
        assert!(identity.map(&Suit::D) == Suit::D);
        assert!(identity.map(&Suit::H) == Suit::H);
        assert!(identity.map(&Suit::S) == Suit::S);
    }

    #[test]
    fn map_arbitrary() {
        let permutation = Permutation([Suit::H, Suit::S, Suit::C, Suit::D]);
        assert!(permutation.map(&Suit::C) == Suit::H);
        assert!(permutation.map(&Suit::D) == Suit::S);
        assert!(permutation.map(&Suit::H) == Suit::C);
        assert!(permutation.map(&Suit::S) == Suit::D);
    }

    #[test]
    fn permute_simple() {
        let permutation = Permutation([Suit::H, Suit::C, Suit::S, Suit::D]);
        let hearts = Hand::from(0b_0100_0100_0100_0100_0100_0100_0100_0100_u64);
        let spades = Hand::from(0b_1000_1000_1000_1000_1000_1000_1000_1000_u64);
        assert!(permutation.image(&hearts) == spades);
    }

    #[test]
    fn permute_unique() {
        let ref hand = Hand::try_from("Ac Kd Qh Js").unwrap();
        let mut unique = std::collections::HashSet::new();
        let n = Permutation::exhaust()
            .into_iter()
            .map(|p| p.image(hand))
            .inspect(|h| assert!(unique.insert(*h)))
            .count();
        assert!(n == 24);
    }

    #[test]
    fn permute_complex() {
        let permutation = Permutation([Suit::D, Suit::H, Suit::C, Suit::S]);
        let original = Hand::from(0b_1010_1010_1010_1010__0100_0100_0100_0100_u64);
        let permuted = Hand::from(0b_1100_1100_1100_1100__0001_0001_0001_0001_u64);
        assert!(permutation.image(&original) == permuted);
    }

    #[test]
    fn permute_rotation() {
        let permutation = Permutation([Suit::S, Suit::C, Suit::D, Suit::H]);
        let original = Hand::try_from("Ac Kd Qh Js").unwrap();
        let permuted = Hand::try_from("As Kc Qd Jh").unwrap();
        assert!(permutation.image(&original) == permuted);
    }

    #[test]
    #[cfg(not(feature = "shortdeck"))]
    fn permute_interior() {
        let permutation = Permutation([Suit::C, Suit::H, Suit::D, Suit::S]);
        let original = Hand::try_from("2c 3d 4h 5s").unwrap();
        let permuted = Hand::try_from("2c 3h 4d 5s").unwrap();
        assert!(permutation.image(&original) == permuted);
    }

    #[test]
    fn permute_identity() {
        use crate::Arbitrary;
        let permutation = Permutation::identity();
        let hand = Hand::random();
        assert!(permutation.image(&hand) == hand);
    }
}
