use super::hand::Hand;
use super::observation::Observation;
use super::street::Street;
use super::suit::Suit;

/// many Observations are strategically equivalent,
/// so we can canonize to reduce the index space of
/// learned Abstractions.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Isomorphism(Observation);

impl Isomorphism {
    pub fn is_canonical(observation: &Observation) -> bool {
        Permutation::identity() == Permutation::from(observation)
    }
    pub fn enumerate(street: Street) -> Vec<Observation> {
        Observation::enumerate(street)
            .into_iter()
            .filter(|o| Self::is_canonical(o))
            .collect()
    }
}

impl From<Observation> for Isomorphism {
    fn from(ref observation: Observation) -> Self {
        let permutation = Permutation::from(observation);
        let secret = permutation.transform(observation.secret());
        let public = permutation.transform(observation.public());
        Self(Observation::from((secret, public)))
    }
}

/// an array of 4 unique Suits represents
/// any of the 4! = 24 possible permutations.
/// by assuming a "canonical" order of suits (C < D < H < S),
/// we map C -> P[0], D -> P[1], H -> P[2], S -> P[3].
#[derive(PartialEq, Eq)]
struct Permutation([Suit; 4]);

impl Permutation {
    fn map(&self, suit: Suit) -> Suit {
        self.0[suit as usize]
    }
    /// this might decompose well into a Suit::all().map(...) impl
    pub fn transform(&self, hand: &Hand) -> Hand {
        let mut result = 0u64;
        for suit in Suit::all() {
            let old = suit;
            let new = self.map(suit);
            let shift = new as i8 - old as i8;
            let overlap = u64::from(*hand) & u64::from(suit);
            let shifted = if shift >= 0 {
                overlap >> (shift as u32)
            } else {
                overlap << (shift.abs() as u32)
            };
            result |= shifted;
        }
        Hand::from(result)
    }

    pub const fn identity() -> Self {
        Self(Suit::all())
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
        suits.sort_by(|a, b| b.cmp(a)); // reverse sort
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
    use super::*;

    #[test]
    fn identity_map() {
        let identity = Permutation::identity();
        assert!(identity.map(Suit::C) == Suit::C);
        assert!(identity.map(Suit::D) == Suit::D);
        assert!(identity.map(Suit::H) == Suit::H);
        assert!(identity.map(Suit::S) == Suit::S);
    }

    #[test]
    fn arbitrary_map() {
        let permutation = Permutation([Suit::H, Suit::S, Suit::C, Suit::D]);
        assert!(permutation.map(Suit::C) == Suit::H);
        assert!(permutation.map(Suit::D) == Suit::S);
        assert!(permutation.map(Suit::H) == Suit::C);
        assert!(permutation.map(Suit::S) == Suit::D);
    }

    #[test]
    fn identity_transform() {
        let identity = Permutation::identity();
        let hand = Hand::random();
        assert!(identity.transform(&hand) == hand);
    }

    #[test]
    fn simple_transform() {
        let permutation = Permutation([Suit::H, Suit::C, Suit::S, Suit::D]);
        let hearts = Hand::from(0b_0100_0100_0100_0100_0100_0100_0100_0100_u64);
        let spades = Hand::from(0b_1000_1000_1000_1000_1000_1000_1000_1000_u64);
        assert!(permutation.transform(&hearts) == spades);
    }

    #[test]
    fn arbitrary_transform() {
        let permutation = Permutation([Suit::D, Suit::H, Suit::C, Suit::S]);
        let original = Hand::from(0b_1010_1010_1010_1010__0100_0100_0100_0100_u64);
        let permuted = Hand::from(0b_1100_1100_1100_1100__0001_0001_0001_0001_u64);
        assert!(permutation.transform(&original) == permuted);
    }
}
