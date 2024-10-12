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
        let permutation = Permutation::from(observation);
        let observation = permutation.transform(observation);
        // print!("{permutation}");
        Self(observation)
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
        let observation = Observation::from(Street::Turn);
        let isomorphism = Isomorphism::from(observation);
        println!("{observation}");
        println!("{isomorphism}");
        Permutation::exhaust()
            .iter()
            .map(|p| p.transform(observation))
            .map(|o| Isomorphism::from(o))
            .inspect(|&i| assert!(isomorphism == i))
            .count();

        // the following cases fail this test
        // something about Observation as Iterator<Suit> not properly handling pairs.

        // 2c2d + 3s4c5sJsKs
        // 2c2d + 3h4d5hJhKh

        // 6dTc + 4c6c7h8h8s
        // 6dTc + 4c6c7s8h8s

        // 7c7h + 5h6c9hJhJs
        // 7c7d + 5c6d9cJcJd

        // 3d7d + 4c7h9dAcAh
        // 3c7c + 4h7d9cAdAh

        // 2h2s + 6c6dJhQhKh
        // 2c2d + 6h6sJdQdKd
    }

    // #[test]
    // fn tricky_1() {
    //     let observation = Observation::from((Hand::from("2c 2d"), Hand::from("3c 4s 5s")));
    //     let isomorphism = Isomorphism::from(observation);
    //     println!("OBS {observation}");
    //     println!("ISO {isomorphism}");
    //     println!();
    //     Permutation::exhaust()
    //         .iter()
    //         .inspect(|_| println!("{observation} ORIGINAL"))
    //         .inspect(|p| print!("{p}"))
    //         .map(|p| p.transform(observation))
    //         .inspect(|o| println!("{o} TRANSFORMED"))
    //         .map(|o| Isomorphism::from(o))
    //         .inspect(|&i| println!("{i} CANONICAL\n"))
    //         .inspect(|&i| assert!(isomorphism == i))
    //         .count();
    // }

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
