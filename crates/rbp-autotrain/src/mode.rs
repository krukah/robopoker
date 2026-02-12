//! Training mode selection from command line arguments.
use crate::*;
use rbp_database::Check;
use rbp_nlhe::NlheProfile;
use rbp_pg::Schema;

/// Training mode parsed from command line arguments
pub enum Mode {
    Status,
    Cluster,
    Fast,
    Slow,
    Reset,
}

impl Mode {
    pub fn from_args() -> Self {
        std::env::args()
            .find_map(|a| match a.as_str() {
                "--cluster" => Some(Self::Cluster),
                "--status" => Some(Self::Status),
                "--fast" => Some(Self::Fast),
                "--slow" => Some(Self::Slow),
                "--reset" => Some(Self::Reset),
                _ => None,
            })
            .unwrap_or_else(|| {
                eprintln!("Usage: trainer --status | --cluster | --fast | --slow | --reset");
                std::process::exit(1);
            })
    }

    pub async fn run() {
        let client = rbp_pg::db().await;
        match Self::from_args() {
            Self::Cluster => PreTraining::run(&client).await,
            Self::Status => client.status().await,
            Self::Fast => FastSession::new(client).await.train().await,
            Self::Slow => SlowSession::new(client).await.train().await,
            Self::Reset => Self::reset(&client).await,
        }
    }
    async fn reset(client: &tokio_postgres::Client) {
        log::info!("Truncating blueprint table...");
        client
            .execute(<NlheProfile as Schema>::truncates(), &[])
            .await
            .expect("truncate blueprint");
        log::info!("Resetting epoch counter...");
        client
            .execute(<EpochMeta as Schema>::truncates(), &[])
            .await
            .expect("reset epoch");
        log::info!("Reset complete.");
    }
}
