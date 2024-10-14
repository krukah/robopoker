use super::hand::Hand;
use super::observation::Observation;
use super::suit::Suit;

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
impl From<Observation> for Permutation {
    fn from(observation: Observation) -> Self {
        let mut permutation = Suit::all();
        let mut colex = Suit::all().map(|suit| Self::colex(&observation, &suit));
        colex.sort_by(|a, b| Self::order(a, b));
        for (i, (suit, _, _)) in colex.into_iter().enumerate() {
            let index = suit as usize;
            let value = Suit::from(i as u8);
            permutation[index] = value;
        }
        Self(permutation)
    }
}

impl Permutation {
    pub const fn identity() -> Self {
        Self(Suit::all())
    }
    pub fn permute(&self, ref observation: Observation) -> Observation {
        Observation::from((
            self.replace(observation.pocket()),
            self.replace(observation.public()),
        ))
    }
    pub fn replace(&self, hand: &Hand) -> Hand {
        Suit::all()
            .iter()
            .map(|suit| self.image(suit, hand))
            .fold(Hand::empty(), |acc, x| Hand::add(acc, x))
    }
    pub fn exhaust() -> [Self; 24] {
        let mut index = 0;
        let mut result = [Self::identity(); 24];
        for a in 0..4 {
            for b in 0..4 {
                for c in 0..4 {
                    let d = 6 - a - b - c;
                    if a == b || b == c || c == a || d > 3 {
                        continue;
                    }
                    result[index] = Self([
                        Suit::from(a as u8),
                        Suit::from(b as u8),
                        Suit::from(c as u8),
                        Suit::from(d as u8),
                    ]);
                    index += 1;
                }
            }
        }
        result
    }

    /// 1. choose preferred pocket v public
    /// 2. choose preferred Ord Rank
    /// 3. choose preferred pocket v public
    /// 4. choose preferred Ord Rank
    /// 5. All else equal, use the arbitrary Suit ordering for tiebreaking
    fn order(a: &(Suit, Hand, Hand), b: &(Suit, Hand, Hand)) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
            .then_with(|| a.1.size().cmp(&b.1.size()))
            .then_with(|| a.2.size().cmp(&b.2.size()))
            .then_with(|| a.1.min_rank().cmp(&b.1.min_rank()))
            .then_with(|| a.2.min_rank().cmp(&b.2.min_rank()))
            .then_with(|| b.0.cmp(&a.0))
    }

    /// there's this thing called co-lexicographic order
    /// which is a total ordering on some sub sets of cards
    /// in our case Observation. it implements Order at different
    /// scopes to break symmetries of strategically identical Observations.
    ///
    fn colex(observation: &Observation, suit: &Suit) -> (Suit, Hand, Hand) {
        let pocket = observation.pocket().of(suit);
        let public = observation.public().of(suit);
        (*suit, pocket, public)
    }

    /// get the image of a Hand under a Permutation
    fn image(&self, suit: &Suit, hand: &Hand) -> Hand {
        let old = *suit;
        let new = self.get(suit);
        let shift = new as i8 - old as i8;
        let cards = u64::from(*suit) & u64::from(*hand);
        if shift >= 0 {
            Hand::from(cards << shift as u64)
        } else {
            Hand::from(cards >> shift.abs() as u64)
        }
    }
    /// get the image of a Suit under a Permutation
    fn get(&self, suit: &Suit) -> Suit {
        self.0[*suit as usize]
    }
    /// get the preimage of a Suit under the Permutation without allocating
    #[allow(dead_code)]
    fn pre(&self, suit: &Suit) -> Suit {
        self.0
            .iter()
            .position(|s| s == suit)
            .map(|i| Suit::from(i as u8))
            .expect("suit in permutation")
    }
}

impl std::fmt::Display for Permutation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Suit::all()
            .iter()
            .map(|s| writeln!(f, "{} -> {}", s, self.get(s)))
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
        assert!(permutation.replace(&hearts) == spades);
    }

    #[test]
    fn permute_unique() {
        let ref hand = Hand::from("Ac Kd Qh Js");
        let mut unique = std::collections::HashSet::new();
        let n = Permutation::exhaust()
            .into_iter()
            .map(|p| p.replace(hand))
            .inspect(|h| assert!(unique.insert(*h)))
            .count();
        assert!(n == 24);
    }

    #[test]
    fn permute_complex() {
        let permutation = Permutation([Suit::D, Suit::H, Suit::C, Suit::S]);
        let original = Hand::from(0b_1010_1010_1010_1010__0100_0100_0100_0100_u64);
        let permuted = Hand::from(0b_1100_1100_1100_1100__0001_0001_0001_0001_u64);
        assert!(permutation.replace(&original) == permuted);
    }

    #[test]
    fn permute_rotation() {
        let permutation = Permutation([Suit::S, Suit::C, Suit::D, Suit::H]);
        let original = Hand::from("Ac Kd Qh Js");
        let permuted = Hand::from("As Kc Qd Jh");
        assert!(permutation.replace(&original) == permuted);
    }

    #[test]
    fn permute_interior() {
        let permutation = Permutation([Suit::C, Suit::H, Suit::D, Suit::S]);
        let original = Hand::from("2c 3d 4h 5s");
        let permuted = Hand::from("2c 3h 4d 5s");
        assert!(permutation.replace(&original) == permuted);
    }

    #[test]
    fn permute_identity() {
        let permutation = Permutation::identity();
        let hand = Hand::random();
        assert!(permutation.replace(&hand) == hand);
    }
}
