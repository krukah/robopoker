//! Fast in-memory training session
use crate::*;
use rbp_database::*;
use rbp_mccfr::*;
use rbp_nlhe::Flagship;
use rbp_nlhe::NlheProfile;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// Fast in-memory training using Pluribus.
pub struct FastSession {
    client: Arc<Client>,
    solver: Flagship,
    /// Epoch count loaded from DB at session start (`Stage::stamp` adds a delta, not an absolute).
    epoch_at_start: usize,
}

impl FastSession {
    #[allow(unreachable_code)]
    pub async fn new(client: Arc<Client>) -> Self {
        PreTraining::run(&client).await;
        let hydrate_started = std::time::Instant::now();
        let solver = Flagship::hydrate(client.clone()).await;
        let hydrate_elapsed = hydrate_started.elapsed();
        log::info!(
            "hydrate (load into memory) took {:.3}s ({} ms)",
            hydrate_elapsed.as_secs_f64(),
            hydrate_elapsed.as_millis()
        );
        log::warn!("terminating after hydrate (load-timing run)");
        std::process::exit(0);
        let epoch_at_start = solver.profile().epochs();
        Self {
            solver,
            client,
            epoch_at_start,
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
        let client = self.client;
        let epochs = self.solver.profile.epochs();
        let delta = epochs.saturating_sub(self.epoch_at_start);
        let profile = self.solver.profile;
        client.stage().await;
        let copy = format!(
            "COPY {t} (past, present, choices, edge, weight, regret, evalue, counts) FROM STDIN BINARY",
            t = rbp_database::STAGING
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
        client.stamp(delta).await;
        log::info!(
            "profile sync complete (epoch {}, +{} this session)",
            epochs,
            delta
        );
    }
}
