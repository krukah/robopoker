#![allow(unused)]

use super::abstraction::Abstraction;
use super::layer::Layer;
use crate::cards::observation::Observation;

pub struct Abstractor;

impl Abstractor {
    pub async fn upload() {
        Layer::outer()
            .await
            .save() // river
            .await
            .inner()
            .save() // turn
            .await
            .inner()
            .save() // flop
            .await
            .inner()
            .save() // preflop
            .await;
    }
    pub async fn download() -> Self {
        todo!("try to load ~1.2TB of Obs -> Abs map into memory, lmao")
    }
    pub fn lookup(&self, observation: Observation) -> Abstraction {
        todo!("hopefully just a simple Hash or BTree lookup")
    }
}
