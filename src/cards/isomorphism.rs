use super::observation::Observation;
use super::permutation::Permutation;
use super::street::Street;

/// because of the equivalence of Suit,
/// many Observations are strategically equivalent !
/// so we can reduce the index space of learned
/// Abstractions by de-symmetrizing over the
/// 4! = 24 Suit Permutation group elements. in other words,
/// canonicalization.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Isomorphism(Observation);

impl Isomorphism {
    pub fn is_canonical(observation: &Observation) -> bool {
        Permutation::from(*observation) == Permutation::identity()
    }
    pub fn exhaust(street: Street) -> Vec<Self> {
        Observation::exhaust(street)
            .into_iter()
            .filter(|o| Self::is_canonical(o))
            .map(|o| Self(o))
            .collect()
    }
}

impl From<Observation> for Isomorphism {
    fn from(observation: Observation) -> Self {
        println!("{}", observation);
        let permutation = Permutation::from(observation);
        let transformed = permutation.transform(observation);
        println!("{}", transformed);
        println!();
        Self(transformed)
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

    #[test]
    fn exhaustive_permutations() {
        let observation = Observation::from(Street::Rive);
        let isomorphism = Isomorphism::from(observation);
        Permutation::exhaust()
            .iter()
            .map(|p| p.transform(observation))
            .map(|o| Isomorphism::from(o))
            .inspect(|&i| assert!(isomorphism == i))
            .count();
    }

    #[test]
    fn symmetric_pocket_pair() {
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
    fn symmetric_public_pair() {
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
