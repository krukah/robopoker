//! Training mode selection from command line arguments.
use crate::*;
use daybook::Check;
use daybook::Schema;
use nlhe::NlheBlueprint;

/// Training mode parsed from command line arguments
pub enum Mode {
    Status,
    Cluster,
    Recluster,
    Fast,
    Slow,
    Reset,
    Forget,
}

impl Mode {
    pub async fn run(self) {
        match self {
            Self::Fast => FastSession::new(daybook::Pool::db().await).await.train().await,
            Self::Slow => SlowSession::new(daybook::Pool::db().await).await.train().await,
            Self::Reset => Self::reset(&*daybook::Pool::db().await).await,
            Self::Forget => Self::forget(&*daybook::Pool::db().await).await,
            Self::Status => daybook::Pool::db().await.status().await,
            Self::Cluster => PreTraining::run(&daybook::Pool::db().await).await,
            Self::Recluster => PreTraining::clear(&*daybook::Pool::db().await).await,
        }
    }

    async fn reset(client: &tokio_postgres::Client) {
        // Drop the blueprint table first so `ensure_all` recreates it with the canonical schema
        // (UNIQUE constraint included).
        tracing::info!("Dropping blueprint table before ensure...");
        client
            .execute(&format!("DROP TABLE IF EXISTS {}", daybook::blueprint()), &[])
            .await
            .expect("drop blueprint");
        crate::Tables::ensure_all(client).await;
        tracing::info!("Truncating blueprint table...");
        client
            .execute(<NlheBlueprint as Schema>::truncates(), &[])
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
        // The backend creates these on first serve, so a fresh database has none to truncate.
        client
            .batch_execute(&format!(
                "DO $$ DECLARE t text; BEGIN \
                 FOREACH t IN ARRAY ARRAY['{}', '{}', '{}', '{}'] LOOP \
                 IF to_regclass(t) IS NOT NULL THEN EXECUTE format('TRUNCATE TABLE %I CASCADE', t); END IF; \
                 END LOOP; END $$;",
                daybook::actions(),
                daybook::players(),
                daybook::hands(),
                daybook::rooms(),
            ))
            .await
            .expect("truncate hand histories");
        tracing::info!("Forget complete.");
    }
}
