use crate::cards::observation::NodeObservation;
use crate::clustering::abstraction::NodeAbstraction;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct Producer {
    tx: Sender<(NodeObservation, NodeAbstraction)>,
    shard: usize,
    rivers: Arc<Vec<NodeObservation>>,
}

impl Producer {
    pub fn new(
        shard: usize,
        tx: Sender<(NodeObservation, NodeAbstraction)>,
        rivers: Arc<Vec<NodeObservation>>,
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
                    let abstraction = NodeAbstraction::from(observation);
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
