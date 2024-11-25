use super::edge::Edge;
use crate::Utility;
use std::collections::BTreeMap;

pub struct Regret(BTreeMap<Edge, Utility>);

impl Regret {
    pub fn inner(&self) -> &BTreeMap<Edge, Utility> {
        &self.0
    }
}

impl From<BTreeMap<Edge, Utility>> for Regret {
    fn from(map: BTreeMap<Edge, Utility>) -> Self {
        Self(map)
    }
}
