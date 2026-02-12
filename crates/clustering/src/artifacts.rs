//! Clustering artifacts produced by a layer.

use super::*;

/// Bundle of outputs from clustering a single street.
///
/// Each layer produces three artifacts that are persisted to the database:
/// - [`Lookup`] — Isomorphism → Abstraction mapping
/// - [`Metric`] — Pairwise EMD distances between abstractions
/// - [`Future`] — Abstraction → Histogram centroids (transition model)
pub struct Artifacts {
    /// The hand-to-bucket assignment table.
    pub lookup: Lookup,
    /// Pairwise distances for use in parent layer's EMD.
    pub metric: Metric,
    /// Cluster centroids for transition modeling.
    pub future: Future,
}

impl From<Lookup> for Artifacts {
    fn from(lookup: Lookup) -> Self {
        Self {
            lookup,
            metric: Metric::default(),
            future: Future::default(),
        }
    }
}

#[cfg(feature = "database")]
impl Artifacts {
    pub async fn stream(self, client: &tokio_postgres::Client) {
        use rbp_database::Streamable;
        self.lookup.stream(client).await;
        self.metric.stream(client).await;
        self.future.stream(client).await;
    }
}
