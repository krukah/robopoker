use super::abstraction::Abstraction;
use crate::cards::{observation::Observation, street::Street};
use consumer::Consumer;
use producer::Producer;
use std::sync::Arc;

pub mod consumer;
pub mod producer;
pub mod progress;

const TASKS: usize = 8;
const RIVERS: usize = 2_809_475_760;
const RIVERS_PER_TASK: usize = RIVERS / TASKS;

pub async fn cluster() {
    let mut tasks = Vec::with_capacity(TASKS);
    let ref observations = Arc::new(Observation::all(Street::Rive));
    let (tx, rx) = tokio::sync::mpsc::channel::<(Observation, Abstraction)>(1024);
    let consumer = Consumer::new(rx).await;
    tasks.push(tokio::spawn(consumer.run()));
    for task in 0..TASKS {
        let tx = tx.clone();
        let observations = observations.clone();
        let producer = Producer::new(task, tx, observations);
        tasks.push(tokio::task::spawn(producer.run()));
    }
    futures::future::join_all(tasks).await;
}
