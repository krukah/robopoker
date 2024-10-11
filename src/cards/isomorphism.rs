use super::observation::Observation;
use super::permutation::Permutation;
use super::street::Street;

/// many Observations are strategically equivalent,
/// so we can canonize to reduce the index space of
/// learned Abstractions.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Isomorphism(Observation);

impl Isomorphism {
    pub fn is_canonical(observation: &Observation) -> bool {
        Permutation::from(*observation) == Permutation::identity()
    }
    pub fn enumerate(street: Street) -> Vec<Observation> {
        Observation::enumerate(street)
            .into_iter()
            .filter(|o| Self::is_canonical(o))
            .collect()
    }
}

impl From<Observation> for Isomorphism {
    fn from(observation: Observation) -> Self {
        Self(Permutation::from(observation).transform(observation))
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
    fn isomorphic_exhaustion() {
        let observation = Observation::from(Street::Rive);
        let isomorphism = Isomorphism::from(observation);
        Permutation::enumerate()
            .iter()
            .map(|p| p.transform(observation))
            .map(|o| Isomorphism::from(o))
            .inspect(|&i| assert!(isomorphism == i))
            .count();
    }

    #[test]
    fn isomorphic_monochrome() {
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
    fn isomorphic_semichrome() {
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
    fn isomorphic_demichrome() {
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
    fn isomorphic_polychrome() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("Ac Kd"),
            Hand::from("Qh Js 9c"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("Ah Kc"),
            Hand::from("Qs Jd 9h"),
        )));
        assert!(a == b);
    }

    #[test]
    fn isomorphic_difference() {
        let a = Isomorphism::from(Observation::from((
            Hand::from("Ac Ks"),
            Hand::from("Qd Js Ts"),
        )));
        let b = Isomorphism::from(Observation::from((
            Hand::from("Ac Ks"),
            Hand::from("Qd Jc Tc"),
        )));
        assert!(a != b);
    }
}
