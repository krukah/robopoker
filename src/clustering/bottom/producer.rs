use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct Producer {
    tx: Sender<(Observation, Abstraction)>,
    shard: usize,
    rivers: Arc<Vec<Observation>>,
}

impl Producer {
    pub fn new(
        shard: usize,
        tx: Sender<(Observation, Abstraction)>,
        rivers: Arc<Vec<Observation>>,
    ) -> Self {
        Self { tx, shard, rivers }
    }

    pub async fn run(self) {
        let n = self.rivers.len() / num_cpus::get();
        let beg = self.shard * n;
        let end = self.shard * n + n;
        for index in beg..end {
            match self.rivers.get(index) {
                None => return,
                Some(observation) => {
                    let abstraction = Abstraction::from(observation);
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
