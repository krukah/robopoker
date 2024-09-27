use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::progress::Progress;
use std::collections::BTreeMap;
use tokio::sync::mpsc::Receiver;

pub struct Consumer {
    input: Receiver<(Observation, Abstraction)>,
    table: BTreeMap<Observation, (Histogram, Abstraction)>,
    // database client : Client
}

impl Consumer {
    pub fn new(input: Receiver<(Observation, Abstraction)>) -> Self {
        let table = BTreeMap::new();
        Self { input, table }
    }

    /// it's actually quite memory expensive to bind the single-abstraction Histogram here in the BTreeMap.
    /// it's about 10GB without, 30GB with.
    /// but it's worth it to maintain the same BTreeMap<Observation, (Histogram, Abstraction)> interface.
    /// especially since this is a one-time equity abstraction cost that we keep in database for future use.
    pub async fn run(mut self) -> BTreeMap<Observation, (Histogram, Abstraction)> {
        let mut progress = Progress::new(2_809_475_760, 2_809_475_760 / 20);
        while let Some((observation, abstraction)) = self.input.recv().await {
            let dirac = Histogram::default().witness(abstraction.clone());
            self.table.insert(observation, (dirac, abstraction));
            progress.tick();
        }
        self.table
    }
}

// TODO
// let's be generic over the Metric type. Implement for Map<Pair, f32> the same way we implement for Map<Observation, (Histogram, Abstraction)>
