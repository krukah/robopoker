#![allow(unused)]

use super::abstraction::Abstraction;
use crate::cards::observation::Observation;

struct Abstractor;

impl Abstractor {
    pub fn abstracted(&self, observation: Observation) -> Abstraction {
        unimplemented!()
    }
}
