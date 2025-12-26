//! Fast in-memory training session

use super::*;
use crate::database::*;
use crate::mccfr::*;
use crate::save::*;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// Fast in-memory training using NlheSolver.
pub struct FastSession {
    client: Arc<Client>,
    solver: NlheSolver,
}

impl FastSession {
    pub async fn new(client: Arc<Client>) -> Self {
        Self {
            solver: NlheSolver::hydrate(client.clone()).await,
            client,
        }
    }
}

#[async_trait::async_trait]
impl Trainer for FastSession {
    fn client(&self) -> &Arc<Client> {
        &self.client
    }

    async fn step(&mut self) {
        self.solver.step();
    }

    async fn epoch(&self) -> usize {
        self.solver.profile().epochs()
    }

    async fn checkpoint(&self) -> Option<String> {
        self.solver.profile().metrics().and_then(|m| m.checkpoint())
    }

    async fn summary(&self) -> String {
        self.solver
            .profile()
            .metrics()
            .map(|m| m.summary())
            .unwrap_or_else(|| "training stopped".to_string())
    }

    async fn sync(self) {
        use crate::save::Row;
        let client = self.client;
        let epochs = self.solver.profile.epochs();
        let profile = self.solver.profile;
        client.stage().await;
        let copy = format!(
            "COPY {t} (past, present, future, edge, policy, regret) FROM STDIN BINARY",
            t = crate::save::STAGING
        );
        let writer = BinaryCopyInWriter::new(
            client.copy_in(&copy).await.expect("copy_in"),
            NlheProfile::columns(),
        );
        futures::pin_mut!(writer);
        for row in profile.rows() {
            row.write(writer.as_mut()).await;
        }
        writer.finish().await.expect("finish stream");
        client.merge().await;
        client.stamp(epochs).await;
        log::info!("profile sync complete (epoch {})", epochs);
    }
}
