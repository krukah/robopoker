use crate::gameplay::*;
use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub edge: Edge,
    pub mass: Probability,
}

impl Decision {
    pub fn normalize(self, denom: Probability) -> Self {
        Self {
            edge: self.edge,
            mass: self.mass / denom,
        }
    }
}
