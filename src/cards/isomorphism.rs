use super::observation::Observation;
use super::street::Street;
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
