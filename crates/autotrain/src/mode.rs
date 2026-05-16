//! Training mode selection from command line arguments.
use crate::*;
use rbp_database::Check;
use rbp_database::Schema;
use rbp_nlhe::NlheProfile;

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
        let client = rbp_database::db().await;
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
                rbp_database::actions(),
                rbp_database::players(),
                rbp_database::hands(),
                rbp_database::rooms(),
            ))
            .await
            .expect("truncate hand histories");
        tracing::info!("Forget complete.");
    }
}
