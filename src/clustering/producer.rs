use crate::cards::observation::CardObservation;
use crate::clustering::abstraction::CardAbstraction;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct Producer {
    tx: Sender<(CardObservation, CardAbstraction)>,
    shard: usize,
    rivers: Arc<Vec<CardObservation>>,
}

impl Producer {
    pub fn new(
        shard: usize,
        tx: Sender<(CardObservation, CardAbstraction)>,
        rivers: Arc<Vec<CardObservation>>,
    ) -> Self {
        Self { tx, shard, rivers }
    }

    pub async fn run(self) {
        let len = self.rivers.len() / num_cpus::get();
        let beg = self.shard * len;
        let end = self.shard * len + len;
        for index in beg..end {
            match self.rivers.get(index) {
                None => return,
                Some(&observation) => {
                    let abstraction = CardAbstraction::from(observation);
                    let observation = observation.clone();
                    self.tx
                        .send((observation, abstraction))
                        .await
                        .expect("channel to be open");
                }
            }
        }
    }
}
