use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;

pub struct Producer {
    tx: Sender<(Observation, Abstraction)>,
    shard: usize,
    rivers: Arc<Vec<Observation>>,
}

impl Producer {
    pub(super) fn new(
        shard: usize,
        tx: Sender<(Observation, Abstraction)>,
        rivers: Arc<Vec<Observation>>,
    ) -> Self {
        Self { tx, shard, rivers }
    }

    pub(super) async fn run(self) {
        let beg = self.shard * super::RIVERS_PER_TASK;
        let end = self.shard * super::RIVERS_PER_TASK + super::RIVERS_PER_TASK;
        for index in beg..end {
            if let Some(observation) = self.rivers.get(index) {
                let abstraction = Abstraction::from(observation);
                let observation = observation.clone();
                self.tx
                    .send((observation, abstraction))
                    .await
                    .expect("channel to be open");
            } else {
                return;
            }
        }
    }
}
