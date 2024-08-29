use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::progress::Progress;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub struct Consumer {
    input: Receiver<(Observation, Abstraction)>,
    table: HashMap<Observation, (Histogram, Abstraction)>,
    // database client : Client
}

impl Consumer {
    pub fn new(input: Receiver<(Observation, Abstraction)>) -> Self {
        let table = HashMap::with_capacity(2_809_475_760);
        Self { input, table }
    }

    pub async fn run(mut self) -> HashMap<Observation, (Histogram, Abstraction)> {
        let mut progress = Progress::new(2_809_475_760);
        while let Some((observation, abstraction)) = self.input.recv().await {
            let histogram = Histogram::witness(Histogram::default(), abstraction.clone());
            self.table.insert(observation, (histogram, abstraction));
            progress.tick();
            // database insert
        }
        self.table
    }
}
