use super::observation::Observation;
use super::permutation::Permutation;

/// because of the equivalence of Suit,
/// many Observations are strategically equivalent !
/// so we can reduce the index space of learned
/// Abstractions by de-symmetrizing over the
/// 4! = 24 Suit Permutation group elements. in other words,
/// canonicalization.
///
/// we do something a bit different from ~the literature~ here.
/// truly lossless isomorphism would distinguish between
/// pocket: Hand(Ac As) , board: Hand( Flop(2c 3c 4c) , Turn(5c) )
/// i.e. strategically consider _which cards came on which streets_.
/// but it's memory/compute/efficient to lump all the board cards together,
/// in a kind of lossy imprefect recall kinda way. so we only care
/// about CardsInYourHand vs CardsOnTheBoard without considering street order.
///
/// but we're able to save quite a bit of space along the way.
/// see [`crate::cards::street::Street::n_isomorphisms`] for a sense of how much.
/// but it's approx 4 (* 5) times smaller, as youd expect for without-replacement
/// sampling on the last two Streets.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Isomorphism(pub Observation);

impl From<Observation> for Isomorphism {
    fn from(ref observation: Observation) -> Self {
        let isomorphism = Permutation::from(observation);
        let transformed = isomorphism.permute(observation);
        Self(transformed)
    }
}

impl From<Isomorphism> for Observation {
    fn from(equivalence: Isomorphism) -> Self {
        equivalence.0
    }
}

impl From<i64> for Isomorphism {
    fn from(i: i64) -> Self {
        Self(Observation::from(i))
    }
}

impl Isomorphism {
    pub fn is_canonical(observation: &Observation) -> bool {
        Permutation::from(observation) == Permutation::identity()
    }
}

impl std::fmt::Display for Isomorphism {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::hand::Hand;
    use crate::cards::permutation::Permutation;
    use crate::cards::street::Street;

    #[test]
    fn false_positives() {
        let observation = Observation::from(Street::Rive);
        let isomorphism = Isomorphism::from(observation);
        assert!(Permutation::exhaust()
            .iter()
            .map(|p| p.permute(&observation))
            .map(|o| Isomorphism::from(o))
            .all(|i| i == isomorphism));
    }

    #[test]
    fn false_negatives() {
        let observation = Observation::from(Street::Rive);
        let isomorphism = Isomorphism::from(observation);
        let transformed = Observation::from(isomorphism);
        assert!(Permutation::exhaust()
            .iter()
            .map(|p| p.permute(&transformed))
            .any(|o| o == observation));
    }

    #[test]
    #[cfg(not(feature = "shortdeck"))]
    fn super_symmetry() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("2s Ks"),
            Hand::from("2d 5h 8c Tc Th"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("2s Ks"),
            Hand::from("2h 5c 8d Tc Td"),
        )));
        assert!(a == b);
    }

    #[test]
    fn pocket_rank_symmetry() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("Ac Ad"),
            Hand::from("Jc Ts 5s"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("As Ah"),
            Hand::from("Js Tc 5c"),
        )));
        assert!(a == b);
    }

    #[test]
    fn public_rank_symmetry() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("Td As"),
            Hand::from("Ts Ks Kh"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("Tc Ad"),
            Hand::from("Td Kd Kh"),
        )));
        assert!(a == b);
    }

    #[test]
    fn offsuit_backdoor() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("As Jh"),
            Hand::from("Ks Js 2d"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("Ah Jd"),
            Hand::from("Kh Jh 2c"),
        )));
        assert!(a == b);
    }

    #[test]
    fn offsuit_draw() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("As Qh"),
            Hand::from("Ks Js 2s"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("Ad Qh"),
            Hand::from("Kd Jd 2d"),
        )));
        assert!(a == b);
    }

    #[test]
    fn monochrome() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("Ad Kd"),
            Hand::from("Qd Jd Td"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("As Ks"),
            Hand::from("Qs Js Ts"),
        )));
        assert!(a == b);
    }

    #[test]
    fn antichrome() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("Ac Kc"),
            Hand::from("Qs Js Ts"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("As Ks"),
            Hand::from("Qh Jh Th"),
        )));
        assert!(a == b);
    }

    #[test]
    fn semichrome() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("Ac Ks"),
            Hand::from("Qc Js Ts"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("Ad Kh"),
            Hand::from("Qd Jh Th"),
        )));
        assert!(a == b);
    }

    #[test]
    fn polychrome() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("Ac Kd"),
            Hand::from("Qh Js 9c"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("Ah Ks"),
            Hand::from("Qc Jd 9h"),
        )));
        assert!(a == b);
    }
}
