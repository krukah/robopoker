//! Pretraining - hierarchical clustering pipeline for poker abstractions.
//!
//! Manages clustering from scratch to postgres without disk I/O:
//! 1. River: equity-based abstractions (computed from scratch)
//! 2. Turn: k-means on river distributions (hydrates river data)
//! 3. Flop: k-means on turn distributions (hydrates turn data)
//! 4. Preflop: 1:1 isomorphism enumeration (computed from scratch)
use rbp_cards::*;
use rbp_clustering::*;
use rbp_database::*;
use rbp_gameplay::*;
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
    /// Always runs finalize to ensure derived tables exist.
    pub async fn run(client: &Arc<Client>) {
        let streets = Self::pending(client).await;
        for street in streets.iter().cloned() {
            log::info!("{:<32}{:<32}", "beginning clustering", street);
            Self::cluster(street, client).await.stream(client).await;
        }
        if streets.len() > 0 {
            Self::index(client).await;
        }
        Self::derive::<Abstraction>(client).await;
        Self::derive::<Street>(client).await;
        log::info!("{:<32}{:<32}", "vacuum analyze", "all tables");
        client
            .batch_execute("VACUUM ANALYZE;")
            .await
            .expect("vacuum analyze");
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

    /// Index tables after data is streamed.
    async fn index(client: &Arc<Client>) {
        Lookup::finalize(client).await;
        Metric::finalize(client).await;
        Future::finalize(client).await;
    }

    /// Derive a table from existing data using SQL functions.
    async fn derive<D>(client: &Arc<Client>)
    where
        D: Derive,
    {
        let absent = client
            .query(
                &format!(
                    "SELECT 1 FROM information_schema.tables WHERE table_name = '{}'",
                    D::name()
                ),
                &[],
            )
            .await
            .map(|rows| rows.is_empty())
            .unwrap_or(true);
        if absent {
            log::info!("{:<32}{:<32}", "creating table", D::name());
            client.batch_execute(D::creates()).await.expect("creates");
        }
        if client
            .query(&format!("SELECT 1 FROM {} LIMIT 1 ", D::name()), &[])
            .await
            .map(|rows| rows.is_empty())
            .unwrap_or(true)
        {
            log::info!("{:<32}{:<32}", "deriving table", D::name());
            client.batch_execute(&D::derives()).await.expect("derives");
            log::info!("{:<32}{:<32}", "indexing table", D::name());
            client.batch_execute(D::indices()).await.expect("indices");
            log::info!("{:<32}{:<32}", "freezing table", D::name());
            client.batch_execute(D::freeze()).await.expect("freeze");
        } else {
            log::info!("{:<32}{:<32}", "table already derived", D::name());
        }
    }
}
