use super::*;

/// Training mode parsed from command line arguments
pub enum Mode {
    Status,
    Cluster,
    Fast,
    Slow,
}

impl Mode {
    pub fn from_args() -> Self {
        std::env::args()
            .find_map(|a| match a.as_str() {
                "--cluster" => Some(Self::Cluster),
                "--status" => Some(Self::Status),
                "--fast" => Some(Self::Fast),
                "--slow" => Some(Self::Slow),
                _ => None,
            })
            .unwrap_or_else(|| {
                eprintln!("Usage: trainer --status | --cluster | --fast | --slow");
                std::process::exit(1);
            })
    }

    pub async fn run() {
        let client = crate::save::db().await;
        match Self::from_args() {
            Self::Cluster => SlowSession::new(client).await.pretraining().await,
            Self::Status => SlowSession::new(client).await.status().await,
            Self::Fast => FastSession::new(client).await.train().await,
            Self::Slow => SlowSession::new(client).await.train().await,
        }
    }
}
