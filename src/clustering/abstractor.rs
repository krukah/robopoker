#![allow(unused)]

use super::abstraction::Abstraction;
use super::layer::Layer;
use crate::cards::observation::Observation;

struct Abstractor;

impl Abstractor {
    pub fn abstracted(&self, observation: Observation) -> Abstraction {
        unimplemented!()
    }

    pub async fn cluster() {
        Layer::outer()
            .await
            .save()
            .await
            .inner()
            .save()
            .await
            .inner()
            .save()
            .await
            .inner()
            .save()
            .await;
    }
}
