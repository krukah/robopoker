use super::edge::Edge;
use crate::Arbitrary;
use crate::Probability;
use std::collections::BTreeMap;

/// probability vector over the simplex of edges
pub struct Policy(BTreeMap<Edge, Probability>);

impl Policy {
    pub fn inner(&self) -> &BTreeMap<Edge, Probability> {
        &self.0
    }
}

impl From<BTreeMap<Edge, Probability>> for Policy {
    fn from(map: BTreeMap<Edge, Probability>) -> Self {
        Self(map)
    }
}

impl Arbitrary for Policy {
    fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(1..=8);
        Self::from(
            (0..n)
                .map(|_| (Edge::random(), rng.gen::<Probability>()))
                .collect::<BTreeMap<Edge, Probability>>(),
        )
    }
}
