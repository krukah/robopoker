use super::edge::Edge;
use super::memory::Memory;
use super::policy::Policy;
use crate::Arbitrary;
use crate::Probability;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Strategy(pub BTreeMap<Edge, Memory>);

impl Strategy {
    pub fn new() -> Self {
        Self(BTreeMap::default())
    }
    pub fn policy(&self) -> Policy {
        Policy::from(
            self.0
                .iter()
                .map(|(edge, memory)| (edge.clone(), memory.policy()))
                .collect::<BTreeMap<Edge, Probability>>(),
        )
    }
    pub fn weight(&self, edge: &Edge) -> Probability {
        let denom = self.0.values().map(|s| s.policy()).sum::<Probability>();
        let numer = self.0.get(edge).expect("edge in infoset").policy();
        numer / denom
    }
    pub fn get(&self, edge: &Edge) -> Option<&Memory> {
        self.0.get(edge)
    }
    pub fn get_mut(&mut self, edge: &Edge) -> Option<&mut Memory> {
        self.0.get_mut(edge)
    }

    pub fn entry(&mut self, edge: Edge) -> std::collections::btree_map::Entry<Edge, Memory> {
        self.0.entry(edge)
    }

    pub fn values(&self) -> std::collections::btree_map::Values<Edge, Memory> {
        self.0.values()
    }

    pub fn iter(&self) -> std::collections::btree_map::Iter<Edge, Memory> {
        self.0.iter()
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Arbitrary for Strategy {
    fn arbitrary() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(1..=8);
        Self(
            (0..n)
                .map(|_| (Edge::arbitrary(), Memory::arbitrary()))
                .collect(),
        )
    }
}
