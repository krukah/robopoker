use super::observation::Observation;
use super::observations::ObservationIterator;
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

impl From<Observation> for Isomorphism {
    fn from(observation: Observation) -> Self {
        let permutation = Permutation::from(observation);
        let transformed = permutation.permute(observation);
        Self(transformed)
    }
}

impl Isomorphism {
    pub fn is_canonical(observation: &Observation) -> bool {
        Permutation::from(*observation) == Permutation::identity()
    }
    pub fn exhaust<'a>(street: Street) -> impl Iterator<Item = Self> + 'a {
        ObservationIterator::from(street)
            .filter(Self::is_canonical)
            .map(Self)
    }
    pub fn children<'a>(&'a self) -> impl Iterator<Item = Self> + 'a {
        self.0
            .children()
            .filter(|o| Self::is_canonical(o))
            .map(|o| Self(o))
    }
    pub fn size(street: Street) -> usize {
        match street {
            Street::Pref => 0_________169,
            Street::Flop => 0___1_286_792,
            Street::Turn => 0__55_190_538,
            Street::Rive => 2_428_287_420,
        }
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
        let mut count = 0;
        loop {
            let observation = Observation::from(Street::Turn);
            let isomorphism = Isomorphism::from(observation);
            let mut result = Ok(());
            let mut error_permutation = None;
            for permutation in Permutation::exhaust() {
                let transformed = Isomorphism::from(permutation.permute(observation));
                if isomorphism != transformed {
                    result = Err((transformed, observation));
                    error_permutation = Some(permutation);
                    break;
                }
            }
            count += 1;
            if let Err((i, obs)) = result {
                println!("assertion failed at {}!", count);
                println!("observation: {}", obs);
                println!("isomorphism: {}", isomorphism);
                println!("transformed: {}", i);
                if let Some(p) = error_permutation {
                    println!("{}", p);
                }
                panic!("");
            }
        }
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

    // #[test]
    // fn n_flop() {
    //     let count = Isomorphism::exhaust(Street::Flop).len();
    //     println!("Number of Flop isomorphisms: {}", count);
    //     assert_eq!(count, 1_286_792);
    // }
    // #[test]
    // fn n_turn() {
    //     let count = Isomorphism::exhaust(Street::Turn).len();
    //     println!("Number of Turn isomorphisms: {}", count);
    //     assert_eq!(count, 55_190_538);
    // }
    // #[test]
    // fn n_river() {
    //     let count = Isomorphism::exhaust(Street::Rive).len();
    //     println!("Number of River isomorphisms: {}", count);
    //     assert_eq!(count, 2_428_287_420);
    // }
}
