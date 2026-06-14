//! Training mode selection from command line arguments.
use crate::*;
use holdem::NlheProfile;
use ledger::Check;
use ledger::Schema;

/// Training mode parsed from command line arguments
pub enum Mode {
    Status,
    Cluster,
    Fast,
    Slow,
    Reset,
    Forget,
}

impl Mode {
    pub async fn run(self) {
        let client = ledger::db().await;
        match self {
            Self::Fast => FastSession::new(client).await.train().await,
            Self::Slow => SlowSession::new(client).await.train().await,
            Self::Reset => Self::reset(&client).await,
            Self::Forget => Self::forget(&client).await,
            Self::Status => client.status().await,
            Self::Cluster => PreTraining::run(&client).await,
        }
    }

    async fn reset(client: &tokio_postgres::Client) {
        // Drop the blueprint table first so `ensure_all` recreates it with
        // the canonical schema (UNIQUE constraint included). If a prior
        // process left the table in a broken state — schema without the
        // unique index, or duplicate rows that block index creation — the
        // ensure step would otherwise panic. Reset is the "make it clean
        // regardless of prior state" mode, so DROP > TRUNCATE is correct.
        tracing::info!("Dropping blueprint table before ensure...");
        client
            .execute(&format!("DROP TABLE IF EXISTS {}", ledger::blueprint()), &[])
            .await
            .expect("drop blueprint");
        crate::ensure_all(client).await;
        tracing::info!("Truncating blueprint table...");
        client
            .execute(<NlheProfile as Schema>::truncates(), &[])
            .await
            .expect("truncate blueprint");
        tracing::info!("Resetting epoch counter...");
        client
            .execute(<EpochMeta as Schema>::truncates(), &[])
            .await
            .expect("reset epoch");
        tracing::info!("Truncating snapshot table...");
        client
            .batch_execute(<Snapshot as Schema>::truncates())
            .await
            .expect("truncate snapshot");
        tracing::info!("Truncating regime fingerprint...");
        client
            .batch_execute(<crate::Fingerprint as Schema>::truncates())
            .await
            .expect("truncate fingerprint");
        tracing::info!("Reset complete.");
    }

    async fn forget(client: &tokio_postgres::Client) {
        tracing::info!("Truncating hand history tables...");
        client
            .batch_execute(&format!(
                "TRUNCATE TABLE {}, {}, {} CASCADE; TRUNCATE TABLE {} CASCADE;",
                ledger::actions(),
                ledger::players(),
                ledger::hands(),
                ledger::rooms(),
            ))
            .await
            .expect("truncate hand histories");
        tracing::info!("Forget complete.");
    }
}
