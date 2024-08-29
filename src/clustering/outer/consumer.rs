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

    /// it's actually quite memory expensive to bind the single-abstraction Histogram here in the HashMap.
    /// it's about 10GB without, 30GB with.
    /// but it's worth it to maintain the same HashMap<Observation, (Histogram, Abstraction)> interface.
    /// especially since this is a one-time equity abstraction cost that we keep in database for future use.
    pub async fn run(mut self) -> HashMap<Observation, (Histogram, Abstraction)> {
        let mut progress = Progress::new(2_809_475_760, 500_000);
        while let Some((observation, abstraction)) = self.input.recv().await {
            let histogram = Histogram::witness(Histogram::default(), abstraction.clone());
            self.table.insert(observation, (histogram, abstraction));
            progress.tick();
            // database insert
        }
        self.table
    }
}
