use super::abstraction::Abstraction;
use super::upper::layer::Layer;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::bottom::consumer::Consumer;
use crate::clustering::bottom::producer::Producer;
use std::sync::Arc;

pub mod consumer;
pub mod producer;
pub mod progress;

pub async fn upload() {
    let cpus = num_cpus::get();
    let mut tasks = Vec::with_capacity(cpus);
    let ref observations = Arc::new(Observation::all(Street::Rive));
    let (tx, rx) = tokio::sync::mpsc::channel::<(Observation, Abstraction)>(1024);
    let consumer = Consumer::new(rx).await;
    tasks.push(tokio::spawn(consumer.run()));
    for task in 0..cpus {
        let tx = tx.clone();
        let observations = observations.clone();
        let producer = Producer::new(task, tx, observations);
        tasks.push(tokio::task::spawn(producer.run()));
    }
    futures::future::join_all(tasks).await;
}

pub async fn download() -> Layer {
    todo!()
}
