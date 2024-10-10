use super::observation::Observation;
use super::street::Street;
use super::suit::Suit;

/// many Observations are strategically equivalent,
/// so we can canonize to reduce the index space of
/// learned Abstractions.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Isomorphism(Observation);

impl Isomorphism {
    fn canonize(_: Observation) -> Self {
        todo!()
    }
}

impl From<Observation> for Isomorphism {
    fn from(o: Observation) -> Self {
        if Self::is_canonical(o) {
            Self(o)
        } else {
            Self::canonize(o)
        }
    }
}

impl Isomorphism {
    pub fn is_canonical(_: Observation) -> bool {
        todo!()
    }
    pub fn enumerate(street: Street) -> Vec<Observation> {
        Observation::enumerate(street)
    }
}

/// an array of 4 unique Suits represents
/// any of the 4! = 24 possible permutations.
/// by assuming a "canonical" order of suits (C < D < H < S),
/// we map C -> P[0], D -> P[1], H -> P[2], S -> P[3].
struct Permutation([Suit; 4]);

impl Permutation {
    fn apply(&self, suit: Suit) -> Suit {
        self.0[suit as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arbitrary() {
        let permutation = Permutation([Suit::H, Suit::S, Suit::C, Suit::D]);
        assert!(permutation.apply(Suit::C) == Suit::H);
        assert!(permutation.apply(Suit::D) == Suit::S);
        assert!(permutation.apply(Suit::H) == Suit::C);
        assert!(permutation.apply(Suit::S) == Suit::D);
    }

    #[test]
    fn identity() {
        let identity = Permutation([Suit::C, Suit::D, Suit::H, Suit::S]);
        assert!(identity.apply(Suit::C) == Suit::C);
        assert!(identity.apply(Suit::D) == Suit::D);
        assert!(identity.apply(Suit::H) == Suit::H);
        assert!(identity.apply(Suit::S) == Suit::S);
    }
}
