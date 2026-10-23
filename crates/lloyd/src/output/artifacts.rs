//! Clustering artifacts produced by a layer.

use super::*;

/// The three outputs of clustering one street, all persisted to the database.
pub struct Artifacts {
    /// The hand-to-bucket assignment table.
    lookup: Lookup,
    /// Pairwise distances for use in parent layer's EMD.
    metric: Metric,
    /// Cluster centroids for transition modeling.
    future: Future,
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
    pub fn new(lookup: Lookup, metric: Metric, future: Future) -> Self {
        Self { lookup, metric, future }
    }

    pub async fn stream(self, client: &tokio_postgres::Client) {
        use daybook::Streamable;
        let Self { lookup, metric, future } = self;
        lookup.stream(client).await;
        metric.stream(client).await;
        future.stream(client).await;
    }
}

impl Artifacts {
    pub fn lookup(&self) -> &Lookup {
        &self.lookup
    }

    pub fn metric(&self) -> &Metric {
        &self.metric
    }

    pub fn future(&self) -> &Future {
        &self.future
    }
}
