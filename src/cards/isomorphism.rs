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
        let canonical = Permutation::from(observation);
        let secret = canonical.permute(observation.secret());
        let public = canonical.permute(observation.public());
        Self(Observation::from((secret, public)))
    }
}
