//! Clustering artifacts produced by a layer.

use super::*;

/// The three outputs of clustering one street, all persisted to the database.
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

#[cfg(feature = "server")]
impl Artifacts {
    pub async fn stream(self, client: &tokio_postgres::Client) {
        use daybook::Streamable;
        self.lookup.stream(client).await;
        self.metric.stream(client).await;
        self.future.stream(client).await;
    }
}
