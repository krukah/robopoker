use super::progress::Progress;
use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::upper::histogram::Histogram;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub struct Consumer {
    rx: Receiver<(Observation, Abstraction)>,
    table: HashMap<Observation, (Histogram, Abstraction)>,
}

impl Consumer {
    pub fn new(rx: Receiver<(Observation, Abstraction)>) -> Self {
        Self {
            rx,
            table: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> HashMap<Observation, (Histogram, Abstraction)> {
        let mut progress = Progress::new(2_809_475_760);
        while let Some((observation, abstraction)) = self.rx.recv().await {
            progress.tick();
            let histogram = Histogram::witness(Histogram::default(), abstraction.clone());
            self.table.insert(observation, (histogram, abstraction));
        }
        self.table
    }
}
