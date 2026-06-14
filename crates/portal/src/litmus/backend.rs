//! Concrete `litmus::Ops` impl wrapping `StrategyAPI` and `TrainingAPI`.

use crate::strategy::StrategyAPI;
use crate::training::TrainingAPI;
use cowboys::{ApiGridUsage, ApiStatus, ApiStrategy, Witness};

pub struct Backend {
    strategy: StrategyAPI,
    training: TrainingAPI,
}

impl Backend {
    pub fn new(strategy: StrategyAPI, training: TrainingAPI) -> Self {
        Self { strategy, training }
    }
}

#[async_trait::async_trait]
impl litmus::Ops for Backend {
    async fn policy(&self, recall: Witness) -> anyhow::Result<Option<ApiStrategy>> {
        self.strategy.policy(recall).await
    }

    async fn grid_usage(&self) -> anyhow::Result<Vec<ApiGridUsage>> {
        self.strategy.grid_usage().await
    }

    async fn status(&self) -> anyhow::Result<ApiStatus> {
        self.training.status().await
    }
}
