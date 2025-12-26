//! Pretraining - hierarchical clustering pipeline for poker abstractions.
//!
//! Manages clustering from scratch to postgres without disk I/O:
//! 1. River: equity-based abstractions (computed from scratch)
//! 2. Turn: k-means on river distributions (hydrates river data)
//! 3. Flop: k-means on turn distributions (hydrates turn data)
//! 4. Preflop: 1:1 isomorphism enumeration (computed from scratch)

use crate::cards::*;
use crate::clustering::*;
use crate::database::*;
use crate::save::*;
use std::sync::Arc;
use tokio_postgres::Client;

type PrefLayer = Layer<{ Street::Pref.k() }, { Street::Pref.n_isomorphisms() }>;
type FlopLayer = Layer<{ Street::Flop.k() }, { Street::Flop.n_isomorphisms() }>;
type TurnLayer = Layer<{ Street::Turn.k() }, { Street::Turn.n_isomorphisms() }>;

/// Zero-sized orchestrator for the clustering pipeline.
/// Encapsulates all clustering logic so Trainer stays clean.
pub struct PreTraining;

impl PreTraining {
    /// Run the complete clustering pipeline if needed.
    /// Returns true if any clustering was performed.
    pub async fn run(client: &Arc<Client>) {
        let streets = Self::pending(client).await;
        for street in streets.iter().cloned() {
            log::info!("{:<32}{:<32}", "beginning clustering", street);
            Self::cluster(street, client).await.stream(client).await;
        }
        if streets.len() > 0 {
            Self::finalize(client).await;
        }
    }
    /// Cluster a street via k-means. Dependencies loaded from postgres.
    /// Dispatches to the appropriate const-generic Layer based on street.
    async fn cluster(street: Street, client: &Arc<Client>) -> Artifacts {
        match street {
            Street::Rive => Artifacts::from(Lookup::grow(street)),
            Street::Turn => TurnLayer::cluster(street, client).await,
            Street::Flop => FlopLayer::cluster(street, client).await,
            Street::Pref => PrefLayer::cluster(street, client).await,
        }
    }

    /// Collect unclustered streets in reverse order (river first).
    async fn pending(client: &Arc<Client>) -> Vec<Street> {
        let mut pending = Vec::new();
        for street in Street::all().iter().rev().cloned() {
            if client.clustered(street).await {
                log::info!("{:<32}{:<32}", "skipping clustering", street);
            } else {
                pending.push(street);
            }
        }
        pending
    }

    /// Prepare tables for streaming (truncate if needed).
    #[allow(unused)]
    async fn truncate(client: &Arc<Client>) {
        client
            .batch_execute(&Metric::truncates())
            .await
            .expect("truncate table metric");
        client
            .batch_execute(&Future::truncates())
            .await
            .expect("truncate table transitions");
        client
            .batch_execute(&Lookup::truncates())
            .await
            .expect("truncate table isomorphism");
    }

    /// Finalize tables after all data is streamed.
    async fn finalize(client: &Arc<Client>) {
        Lookup::finalize(client).await;
        Metric::finalize(client).await;
        Future::finalize(client).await;
    }
}
