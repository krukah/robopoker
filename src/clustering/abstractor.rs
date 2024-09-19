#![allow(unused)]

use super::abstraction::Abstraction;
use super::layer::Layer;
use crate::cards::observation::Observation;

pub struct Abstractor;

impl Abstractor {
    pub fn abstracted(&self, observation: Observation) -> Abstraction {
        unimplemented!()
    }

    pub async fn download() -> Self {
        unimplemented!()
    }

    pub async fn upload() {
        return;
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
