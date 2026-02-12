use rbp_cards::*;
use rbp_pg::*;
use std::sync::Arc;
use tokio_postgres::Client;

/// Check defines status queries for training orchestration.
/// Consolidates existence/count checks used by Trainer and PreTraining.
#[async_trait::async_trait]
pub trait Check: Send + Sync {
    async fn epochs(&self) -> usize;
    async fn blueprint(&self) -> usize;
    async fn clustered(&self, street: Street) -> bool;
    async fn status(&self) {
        fn commas(n: usize) -> String {
            n.to_string()
                .as_bytes()
                .rchunks(3)
                .rev()
                .map(|c| std::str::from_utf8(c).unwrap())
                .collect::<Vec<_>>()
                .join(",")
        }
        log::info!("┌────────────┬───────────────┐");
        log::info!("│ Street     │ Clustered     │");
        log::info!("├────────────┼───────────────┤");
        for street in Street::all().iter().rev().cloned() {
            let done = self.clustered(street).await;
            let mark = if done { "✓" } else { " " };
            log::info!(
                "│ {:?}{} │       {}       │",
                street,
                " ".repeat(10 - format!("{:?}", street).len()),
                mark
            );
        }
        log::info!("├────────────┼───────────────┤");
        log::info!("│ Epoch      │ {:>13} │", commas(self.epochs().await));
        log::info!("│ Blueprint  │ {:>13} │", commas(self.blueprint().await));
        log::info!("└────────────┴───────────────┘");
    }
}

#[async_trait::async_trait]
impl Check for Client {
    async fn epochs(&self) -> usize {
        let sql = format!("SELECT value FROM {t} WHERE key = 'current'", t = EPOCH);
        self.query_opt(&sql, &[])
            .await
            .ok()
            .flatten()
            .map(|r| r.get::<_, i64>(0) as usize)
            .unwrap_or(0)
    }
    async fn blueprint(&self) -> usize {
        let sql = format!("SELECT COUNT(*) FROM {t}", t = BLUEPRINT);
        self.query_opt(&sql, &[])
            .await
            .ok()
            .flatten()
            .map(|r| r.get::<_, i64>(0) as usize)
            .unwrap_or(0)
    }
    async fn clustered(&self, street: Street) -> bool {
        let sql = format!("SELECT 1 FROM {t} WHERE obs = $1", t = ISOMORPHISM);
        let obs = i64::from(Isomorphism::from(Observation::from(street)));
        self.query_opt(&sql, &[&obs]).await.ok().flatten().is_some()
    }
}

#[async_trait::async_trait]
impl Check for Arc<Client> {
    async fn epochs(&self) -> usize {
        self.as_ref().epochs().await
    }
    async fn blueprint(&self) -> usize {
        self.as_ref().blueprint().await
    }
    async fn clustered(&self, street: Street) -> bool {
        self.as_ref().clustered(street).await
    }
}
