//! Pretraining - hierarchical clustering pipeline for poker abstractions.
use daybook::*;
use deuce::*;
use kicker::*;
use lloyd::*;
use std::sync::Arc;
use tokio_postgres::Client;

type PrefLayer = Layer<{ Street::Pref.k() }, { Street::Pref.n_isomorphisms() }, Bounds<{ Street::Pref.k() }>>;
type FlopLayer = Layer<{ Street::Flop.k() }, { Street::Flop.n_isomorphisms() }, Bounds<{ Street::Flop.k() }>>;
// Elkan bounds on the turn would be N·K floats (~14.5 GB) to prune a distance that is already cheap.
type TurnLayer = Layer<{ Street::Turn.k() }, { Street::Turn.n_isomorphisms() }, Hamerly<{ Street::Turn.k() }>>;

/// Zero-sized orchestrator for the clustering pipeline.
pub struct PreTraining;

impl PreTraining {
    /// Run the complete clustering pipeline if needed.
    /// Always runs finalize to ensure derived tables exist.
    pub async fn run(client: &Arc<Client>) {
        let streets = Self::pending(client).await;
        for street in streets.iter().copied() {
            tracing::info!(%street, "beginning clustering");
            Self::cluster(street, client).await.stream(client).await;
        }
        if !streets.is_empty() {
            Self::index(client).await;
        }
        Self::derive::<Abstraction>(client).await;
        Self::derive::<Street>(client).await;
        tracing::info!("vacuum analyze all tables");
        client.batch_execute("VACUUM ANALYZE;").await.expect("vacuum analyze");
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
        for street in Street::all().iter().rev().copied() {
            if client.clustered(street).await {
                tracing::info!(%street, "skipping clustering");
            } else {
                pending.push(street);
            }
        }
        pending
    }

    /// Drop every clustering table so the next [`run`](Self::run) rebuilds the abstraction from
    /// scratch.
    pub async fn clear(client: &Client) {
        tracing::warn!("recluster — dropping clustering tables for a full rebuild");
        client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS {}, {}, {}, {}, {} CASCADE;",
                daybook::abstraction(),
                daybook::isomorphism(),
                daybook::street(),
                daybook::transitions(),
                daybook::metric(),
            ))
            .await
            .expect("clear clustering tables");
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
        D: Derive + Send,
    {
        client.ensure::<D>().await;
        if client
            .query(&format!("SELECT 1 FROM {} LIMIT 1 ", D::name()), &[])
            .await
            .as_ref()
            .map_or(true, Vec::is_empty)
        {
            tracing::info!(table = D::name(), "deriving table");
            client.batch_execute(&D::derives()).await.expect("derives");
            tracing::info!(table = D::name(), "indexing table");
            client.batch_execute(D::indices()).await.expect("indices");
            tracing::info!(table = D::name(), "freezing table");
            client.batch_execute(D::freeze()).await.expect("freeze");
        } else {
            tracing::info!(table = D::name(), "table already derived");
        }
    }
}
