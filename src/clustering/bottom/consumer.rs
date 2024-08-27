use super::progress::Progress;
use crate::clustering::bottom::Abstraction;
use crate::clustering::bottom::Observation;
use crate::clustering::upper::histogram::Histogram;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub struct Consumer {
    rx: Receiver<(Observation, Abstraction)>,
    table: HashMap<Observation, (Histogram, Abstraction)>,
}

impl Consumer {
    pub async fn new(rx: Receiver<(Observation, Abstraction)>) -> Self {
        Self {
            rx,
            table: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> HashMap<Observation, (Histogram, Abstraction)> {
        let mut progress = Progress::new();
        while let Some((observation, abstraction)) = self.rx.recv().await {
            progress.tick();
            let histogram = Histogram::witness(Histogram::default(), abstraction.clone());
            self.table.insert(observation, (histogram, abstraction));
        }
        self.table
    }
}
