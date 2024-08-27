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

/// See if the consumer can return an owned Layer.
/// We could make all this ::upload() logic directly part of the Layer impl.
/// This could preserve multithreading while also avoiding the need for async persistence.
/// The only contract we need is to return a Layer in download().
pub async fn upload() {
    let n = num_cpus::get();
    let mut tasks = Vec::with_capacity(n);
    let ref observations = Arc::new(Observation::all(Street::Rive));
    let (tx, rx) = tokio::sync::mpsc::channel::<(Observation, Abstraction)>(1024);
    let consumer = tokio::spawn(Consumer::new(rx).await.run());
    for task in 0..n {
        let tx = tx.clone();
        let observations = observations.clone();
        let producer = tokio::spawn(Producer::new(task, tx, observations).run());
        tasks.push(producer);
    }
    futures::future::join_all(tasks).await;
    consumer.await.expect("consumer task completes");
}

pub async fn download() -> Layer {
    todo!()
}
