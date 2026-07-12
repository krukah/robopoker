//! Fast in-memory training session
use crate::*;
use daybook::*;
use mccfr::*;
use nlhe::Flagship;
use nlhe::NlheProfile;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tokio_postgres::Client;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

/// Fast in-memory training using Pluribus.
pub struct FastSession {
    client: Arc<Client>,
    solver: Flagship,
    flushed: Mutex<Instant>,
    exploit: Mutex<pokerkit::Utility>,
    started: Instant,
}

impl FastSession {
    pub async fn new(client: Arc<Client>) -> Self {
        PreTraining::run(&client).await;
        crate::ensure_all(&client).await;
        Fingerprint::check(&client).await;
        Self {
            solver: Flagship::hydrate(client.clone()).await,
            client,
            flushed: Mutex::new(Instant::now()),
            exploit: Mutex::new(0.),
            started: Instant::now(),
        }
    }

    async fn snapshot(&self) {
        let profile = self.solver.profile();
        let epochs = profile.t();
        let infos = profile.encounters_ref().len() as i64;
        let nodes = profile.encounters_ref().values().map(|e| e.len() as i64).sum::<i64>();
        let exploit = *self.exploit.lock().expect("poison");
        let elapsed = self.started.elapsed().as_secs() as i64;
        let stamped = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_secs() as i64;
        self.client.stage().await;
        let copy = format!(
            "COPY {t} (past, present, choices, edge, weight, regret, payoff, visits) FROM STDIN BINARY",
            t = daybook::staging()
        );
        let writer =
            BinaryCopyInWriter::new(self.client.copy_in(&copy).await.expect("copy_in"), NlheProfile::columns());
        futures::pin_mut!(writer);
        for row in self.solver.profile().rows() {
            row.write(writer.as_mut()).await;
        }
        writer.finish().await.expect("finish stream");
        self.client.merge().await;
        self.client.stamp(epochs).await;
        self.client
            .snapshot(epochs as i64, infos, nodes, exploit, elapsed, stamped)
            .await;
        tracing::info!(epoch = epochs, "profile sync complete");
    }
}

#[async_trait::async_trait]
impl Trainer for FastSession {
    fn client(&self) -> &Arc<Client> {
        &self.client
    }

    fn session_type(&self) -> &'static str {
        "fast"
    }

    async fn step(&mut self) {
        self.solver.step();
    }

    async fn epoch(&self) -> usize {
        self.solver.profile().t()
    }

    async fn checkpoint(&self) -> Option<mccfr::Checkpoint> {
        self.solver.profile().metrics().and_then(mccfr::Metrics::checkpoint)
    }

    async fn summary(&self) -> String {
        self.solver
            .profile()
            .metrics()
            .map_or_else(|| "training stopped".to_string(), Progress::summary)
    }

    async fn flush(&mut self) {
        let due = {
            let flushed = self.flushed.lock().expect("poison");
            flushed.elapsed() >= TrainingHyperParams::get().flush_interval()
        };
        if !due {
            return;
        }
        *self.flushed.lock().expect("poison") = Instant::now();
        let e = self.solver.profile().sum_regret();
        *self.exploit.lock().expect("poison") = e;
        tracing::info!(exploit = e, "exploitability");
        tracing::info!("periodic flush starting...");
        let labels = [
            vitals::KeyValue::new("session_type", self.session_type()),
            vitals::KeyValue::new("regime", format!("{}", pokerkit::regime())),
        ];
        let m = vitals::metrics::get();
        m.mccfr_sum_regret.record(e as f64, &labels);
        let t0 = Instant::now();
        self.snapshot().await;
        m.mccfr_flush_duration_ms
            .record(t0.elapsed().as_secs_f64() * 1000.0, &labels);
    }

    async fn sync(self) {
        self.snapshot().await;
    }
}
