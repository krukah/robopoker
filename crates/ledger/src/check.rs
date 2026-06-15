use super::*;
use deuce::*;
use std::sync::Arc;
use tokio_postgres::Client;

async fn count(client: &Client, table: &str) -> usize {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    measure("check.count", client.query_opt(&sql, &[]))
        .await
        .ok()
        .flatten()
        .map_or(0, |r| r.get::<_, i64>(0) as usize)
}
/// Check defines status queries for training orchestration.
/// Consolidates existence/count checks used by Trainer and PreTraining.
#[async_trait::async_trait]
pub trait Check: Send + Sync {
    async fn epochs(&self) -> usize;
    async fn blueprint(&self) -> usize;
    async fn histories(&self) -> (usize, usize, usize);
    async fn clustered(&self, street: Street) -> bool;
    async fn latest_snapshot(&self) -> Option<(i64, i64, i64, Option<f32>, i64, i64)>;
    async fn status(&self) {
        fn commas(n: usize) -> String {
            n.to_string()
                .as_bytes()
                .rchunks(3)
                .rev()
                .map(std::str::from_utf8)
                .map(Result::unwrap)
                .collect::<Vec<_>>()
                .join(",")
        }
        let mut out = String::new();
        out.push_str("\n┌────────────┬───────────────┐");
        out.push_str("\n│ Street     │ Clustered     │");
        out.push_str("\n├────────────┼───────────────┤");
        for street in Street::all().iter().rev().copied() {
            let done = self.clustered(street).await;
            let mark = if done { "✓" } else { " " };
            out.push_str(&format!(
                "\n│ {:?}{} │       {}       │",
                street,
                " ".repeat(10 - format!("{street:?}").len()),
                mark
            ));
        }
        out.push_str("\n├────────────┼───────────────┤");
        out.push_str(&format!("\n│ Epoch      │ {:>13} │", commas(self.epochs().await)));
        out.push_str(&format!("\n│ Blueprint  │ {:>13} │", commas(self.blueprint().await)));
        out.push_str("\n├────────────┼───────────────┤");
        let (hands, players, actions) = self.histories().await;
        out.push_str(&format!("\n│ Hands      │ {:>13} │", commas(hands)));
        out.push_str(&format!("\n│ Players    │ {:>13} │", commas(players)));
        out.push_str(&format!("\n│ Actions    │ {:>13} │", commas(actions)));
        out.push_str("\n└────────────┴───────────────┘");
        if let Some((epoch, infos, nodes, exploit, elapsed, _stamped)) = self.latest_snapshot().await {
            out.push_str("\n┌────────────────────────────────┐");
            out.push_str("\n│ Latest Snapshot                │");
            out.push_str("\n├────────────┬───────────────────┤");
            out.push_str(&format!("\n│ Epoch      │ {:>17} │", commas(epoch as usize)));
            out.push_str(&format!("\n│ Infos      │ {:>17} │", commas(infos as usize)));
            out.push_str(&format!("\n│ Nodes      │ {:>17} │", commas(nodes as usize)));
            out.push_str(&format!(
                "\n│ Exploit    │ {:>17} │",
                exploit.map_or_else(|| "N/A".to_string(), |e| format!("{e:.6}"))
            ));
            out.push_str(&format!("\n│ Elapsed    │ {:>15}s │", commas(elapsed as usize)));
            out.push_str("\n└────────────┴───────────────────┘");
        }
        tracing::info!("{out}");
    }
}

#[async_trait::async_trait]
impl Check for Client {
    async fn epochs(&self) -> usize {
        let sql = format!("SELECT value FROM {t} WHERE key = 'current'", t = epoch());
        measure("check.epochs", self.query_opt(&sql, &[]))
            .await
            .ok()
            .flatten()
            .map_or(0, |r| r.get::<_, i64>(0) as usize)
    }

    async fn blueprint(&self) -> usize {
        count(self, blueprint()).await
    }

    async fn histories(&self) -> (usize, usize, usize) {
        (count(self, hands()).await, count(self, players()).await, count(self, actions()).await)
    }

    async fn clustered(&self, street: Street) -> bool {
        let sql = format!("SELECT 1 FROM {t} WHERE obs = $1", t = isomorphism());
        let obs = i64::from(Isomorphism::from(Observation::from(street)));
        measure("check.clustered", self.query_opt(&sql, &[&obs]))
            .await
            .ok()
            .flatten()
            .is_some()
    }

    async fn latest_snapshot(&self) -> Option<(i64, i64, i64, Option<f32>, i64, i64)> {
        let sql = format!(
            "SELECT epoch, infos, nodes, exploit, elapsed, stamped FROM {t} ORDER BY id DESC LIMIT 1",
            t = snapshot()
        );
        measure("check.latest_snapshot", self.query_opt(&sql, &[]))
            .await
            .ok()
            .flatten()
            .map(|r| (r.get(0), r.get(1), r.get(2), r.get(3), r.get(4), r.get(5)))
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

    async fn histories(&self) -> (usize, usize, usize) {
        self.as_ref().histories().await
    }

    async fn clustered(&self, street: Street) -> bool {
        self.as_ref().clustered(street).await
    }

    async fn latest_snapshot(&self) -> Option<(i64, i64, i64, Option<f32>, i64, i64)> {
        self.as_ref().latest_snapshot().await
    }
}
