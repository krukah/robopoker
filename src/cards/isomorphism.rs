use super::observation::Observation;
use super::permutation::Permutation;
use crate::Arbitrary;

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

impl From<Isomorphism> for i64 {
    fn from(isomorphism: Isomorphism) -> i64 {
        isomorphism.0.into()
    }
}

impl Arbitrary for Isomorphism {
    fn random() -> Self {
        Self::from(Observation::random())
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
            Hand::try_from("2s Ks").unwrap(),
            Hand::try_from("2d 5h 8c Tc Th").unwrap(),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::try_from("2s Ks").unwrap(),
            Hand::try_from("2h 5c 8d Tc Td").unwrap(),
        )));
        assert!(a == b);
    }

    #[test]
    fn pocket_rank_symmetry() {
        let a = Isomorphism::from(Observation::from((
            Hand::try_from("Ac Ad").unwrap(),
            Hand::try_from("Jc Ts 5s").unwrap(),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::try_from("As Ah").unwrap(),
            Hand::try_from("Js Tc 5c").unwrap(),
        )));
        assert!(a == b);
    }

    #[test]
    fn public_rank_symmetry() {
        let a = Isomorphism::from(Observation::from((
            Hand::try_from("Td As").unwrap(),
            Hand::try_from("Ts Ks Kh").unwrap(),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::try_from("Tc Ad").unwrap(),
            Hand::try_from("Td Kd Kh").unwrap(),
        )));
        assert!(a == b);
    }

    #[test]
    fn offsuit_backdoor() {
        let a = Isomorphism::from(Observation::from((
            Hand::try_from("As Jh").unwrap(),
            Hand::try_from("Ks Js 2d").unwrap(),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::try_from("Ah Jd").unwrap(),
            Hand::try_from("Kh Jh 2c").unwrap(),
        )));
        assert!(a == b);
    }

    #[test]
    fn offsuit_draw() {
        let a = Isomorphism::from(Observation::from((
            Hand::try_from("As Qh").unwrap(),
            Hand::try_from("Ks Js 2s").unwrap(),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::try_from("Ad Qh").unwrap(),
            Hand::try_from("Kd Jd 2d").unwrap(),
        )));
        assert!(a == b);
    }

    #[test]
    fn monochrome() {
        let a = Isomorphism::from(Observation::from((
            Hand::try_from("Ad Kd").unwrap(),
            Hand::try_from("Qd Jd Td").unwrap(),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::try_from("As Ks").unwrap(),
            Hand::try_from("Qs Js Ts").unwrap(),
        )));
        assert!(a == b);
    }

    #[test]
    fn antichrome() {
        let a = Isomorphism::from(Observation::from((
            Hand::try_from("Ac Kc").unwrap(),
            Hand::try_from("Qs Js Ts").unwrap(),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::try_from("As Ks").unwrap(),
            Hand::try_from("Qh Jh Th").unwrap(),
        )));
        assert!(a == b);
    }

    #[test]
    fn semichrome() {
        let a = Isomorphism::from(Observation::from((
            Hand::try_from("Ac Ks").unwrap(),
            Hand::try_from("Qc Js Ts").unwrap(),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::try_from("Ad Kh").unwrap(),
            Hand::try_from("Qd Jh Th").unwrap(),
        )));
        assert!(a == b);
    }

    #[test]
    fn polychrome() {
        let a = Isomorphism::from(Observation::from((
            Hand::try_from("Ac Kd").unwrap(),
            Hand::try_from("Qh Js 9c").unwrap(),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::try_from("Ah Ks").unwrap(),
            Hand::try_from("Qc Jd 9h").unwrap(),
        )));
        assert!(a == b);
    }
}
