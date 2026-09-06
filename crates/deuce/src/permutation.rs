use super::hand::Hand;
use super::observation::Observation;
use super::suit::Suit;
use pokerkit::Arbitrary;

/// A suit relabeling from the symmetric group S₄ — the 24 elements form a
/// group under composition, and applying one puts an observation in canonical
/// isomorphic form.
///
/// Stored as `[Suit; 4]` where index `i` is the image of the `i`-th suit, so
/// `[H, S, C, D]` means C→H, D→S, H→C, S→D.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Permutation([Suit; 4]);

/// Yields a consistent — though possibly non-unique — Permutation mapping an
/// Observation to its canonical form. Suits sort co-lexicographically by how
/// many cards carry them in the hole and on the board, ties broken by the
/// arbitrary `Ord` on Suit.
impl From<&Observation> for Permutation {
    fn from(observation: &Observation) -> Self {
        let mut permutation = Suit::all();
        let mut colex = Suit::all().map(|suit| Self::colex(observation, &suit));
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
    /// Relabels both hole and board cards through the suit mapping.
    pub fn permute(&self, observation: Observation) -> Observation {
        Observation::from((self.image(observation.pocket()), self.image(observation.public())))
    }
    /// Maps every card's suit through the permutation, ranks preserved.
    pub fn image(&self, hand: &Hand) -> Hand {
        Suit::all()
            .iter()
            .map(|suit| self.shift(suit, hand))
            .fold(Hand::empty(), Hand::add)
    }
    /// The inverse: if `self.map(s) = t`, then `self.inverse().map(t) = s`.
    pub fn inverse(&self) -> Self {
        let mut inv = [Suit::C; 4];
        Suit::all().iter().for_each(|s| inv[self.map(s) as usize] = *s);
        Self(inv)
    }
    /// Co-lexicographic ordering on suits: pocket count, board count, min
    /// pocket rank, min board rank, max pocket rank, max board rank, then the
    /// suit enum order as tiebreaker.
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
    /// Co-lexicographic order is a total ordering over subsets of cards — here
    /// Observations — applied at several scopes to break the symmetries
    /// between strategically identical Observations.
    fn colex(observation: &Observation, suit: &Suit) -> (Suit, Hand, Hand) {
        let pocket = observation.pocket().of(suit);
        let public = observation.public().of(suit);
        (*suit, pocket, public)
    }
    /// Filter the hand to the "old" suit, then bitshift it into the "new"
    /// suit, e.g. Full Hand -> Hearts Hand -> Spades Hand.
    fn shift(&self, suit: &Suit, hand: &Hand) -> Hand {
        let old = *suit;
        let new = self.map(suit);
        let shift = new as i8 - old as i8;
        let cards = u64::from(*suit) & u64::from(*hand);
        if shift >= 0 {
            Hand::from(cards << shift as u64)
        } else {
            Hand::from(cards >> shift.unsigned_abs() as u64)
        }
    }
    /// Maps a suit through the permutation.
    pub fn map(&self, suit: &Suit) -> Suit {
        self.0[*suit as usize]
    }
    /// The identity permutation (no change).
    pub const fn identity() -> Self {
        Self(Suit::all())
    }
    /// All 24 permutations of the symmetric group S₄.
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
    use super::super::hand::Hand;
    use super::super::suit::Suit;
    use super::*;

    #[test]
    fn map_identity() {
        let identity = Permutation::identity();
        assert_eq!(identity.map(&Suit::C), Suit::C);
        assert_eq!(identity.map(&Suit::D), Suit::D);
        assert_eq!(identity.map(&Suit::H), Suit::H);
        assert_eq!(identity.map(&Suit::S), Suit::S);
    }

    #[test]
    fn map_arbitrary() {
        let permutation = Permutation([Suit::H, Suit::S, Suit::C, Suit::D]);
        assert_eq!(permutation.map(&Suit::C), Suit::H);
        assert_eq!(permutation.map(&Suit::D), Suit::S);
        assert_eq!(permutation.map(&Suit::H), Suit::C);
        assert_eq!(permutation.map(&Suit::S), Suit::D);
    }

    #[test]
    fn permute_simple() {
        let permutation = Permutation([Suit::H, Suit::C, Suit::S, Suit::D]);
        let hearts = Hand::from(0b_0100_0100_0100_0100_0100_0100_0100_0100_u64);
        let spades = Hand::from(0b_1000_1000_1000_1000_1000_1000_1000_1000_u64);
        assert_eq!(permutation.image(&hearts), spades);
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
        assert_eq!(n, 24);
    }

    #[test]
    fn permute_complex() {
        let permutation = Permutation([Suit::D, Suit::H, Suit::C, Suit::S]);
        let original = Hand::from(0b_1010_1010_1010_1010__0100_0100_0100_0100_u64);
        let permuted = Hand::from(0b_1100_1100_1100_1100__0001_0001_0001_0001_u64);
        assert_eq!(permutation.image(&original), permuted);
    }

    #[test]
    fn permute_rotation() {
        let permutation = Permutation([Suit::S, Suit::C, Suit::D, Suit::H]);
        let original = Hand::try_from("Ac Kd Qh Js").unwrap();
        let permuted = Hand::try_from("As Kc Qd Jh").unwrap();
        assert_eq!(permutation.image(&original), permuted);
    }

    #[test]
    #[cfg(not(feature = "shortdeck"))]
    fn permute_interior() {
        let permutation = Permutation([Suit::C, Suit::H, Suit::D, Suit::S]);
        let original = Hand::try_from("2c 3d 4h 5s").unwrap();
        let permuted = Hand::try_from("2c 3h 4d 5s").unwrap();
        assert_eq!(permutation.image(&original), permuted);
    }

    #[test]
    fn permute_identity() {
        use pokerkit::Arbitrary;
        let permutation = Permutation::identity();
        let hand = Hand::random();
        assert_eq!(permutation.image(&hand), hand);
    }

    #[test]
    fn inverse_identity() {
        let identity = Permutation::identity();
        assert_eq!(identity.inverse(), identity);
    }

    #[test]
    fn inverse_round_trip() {
        for perm in Permutation::exhaust() {
            for suit in Suit::all() {
                let mapped = perm.map(&suit);
                let recovered = perm.inverse().map(&mapped);
                assert_eq!(recovered, suit, "p(s)=t implies p^-1(t)=s");
            }
        }
    }

    #[test]
    fn inverse_involution() {
        for perm in Permutation::exhaust() {
            assert_eq!(perm.inverse().inverse(), perm, "(p^-1)^-1 = p");
        }
    }
}
