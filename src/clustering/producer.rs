use crate::cards::observation::Observation;
use crate::clustering::histogram::Histogram;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct Producer {
    tx: Sender<(Observation, Histogram)>,
    shard: usize,
    turns: Arc<Vec<Observation>>,
}

impl Producer {
    pub fn new(
        shard: usize,
        tx: Sender<(Observation, Histogram)>,
        turns: Arc<Vec<Observation>>,
    ) -> Self {
        Self { tx, shard, turns }
    }

    pub async fn run(self) {
        let len = self.turns.len() / num_cpus::get();
        let beg = self.shard * len;
        let end = self.shard * len + len;
        for index in beg..end {
            match self.turns.get(index) {
                None => return,
                Some(observation) => {
                    self.tx
                        .send((observation.clone(), Histogram::from(observation.clone())))
                        .await
                        .expect("channel to be open");
                }
            }
        }
    }
}
