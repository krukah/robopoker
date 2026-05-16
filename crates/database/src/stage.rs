use super::*;
use rbp_core::Utility;
use std::sync::Arc;
use tokio_postgres::Client;

/// Stage defines bulk upload operations for fast training.
/// Manages staging table lifecycle and batch epoch updates.
#[async_trait::async_trait]
pub trait Stage: Send + Sync {
    async fn stage(&self);
    async fn merge(&self);
    async fn stamp(&self, n: usize);
    async fn snapshot(
        &self,
        epoch: i64,
        infos: i64,
        nodes: i64,
        exploit: Utility,
        elapsed: i64,
        stamped: i64,
    );
}

#[async_trait::async_trait]
impl Stage for Client {
    async fn stage(&self) {
        let sql = format!(
            "DROP   TABLE IF EXISTS {t2};
             CREATE UNLOGGED TABLE  {t2} (LIKE {t1});",
            t1 = blueprint(),
            t2 = staging()
        );
        measure("stage.create", self.batch_execute(&sql))
            .await
            .expect("create staging");
    }

    async fn merge(&self) {
        let sql = format!(
            "INSERT INTO   {t1} (past, present, choices, geometry, edge, weight, regret, payoff, visits)
             SELECT              past, present, choices, geometry, edge, weight, regret, payoff, visits FROM {t2}
             ON CONFLICT  (past, present, choices, geometry, edge)
             DO UPDATE SET
                 weight = EXCLUDED.weight,
                 regret = EXCLUDED.regret,
                 payoff = EXCLUDED.payoff,
                 visits = EXCLUDED.visits;
             DROP TABLE    {t2};",
            t1 = blueprint(),
            t2 = staging()
        );
        measure("stage.merge", self.batch_execute(&sql))
            .await
            .expect("upsert blueprint");
    }

    async fn stamp(&self, n: usize) {
        let sql = format!(
            "UPDATE {t} SET value = $1 WHERE key = 'current'",
            t = epoch()
        );
        measure("stage.stamp", self.execute(&sql, &[&(n as i64)]))
            .await
            .expect("update epoch");
    }

    async fn snapshot(
        &self,
        epoch: i64,
        infos: i64,
        nodes: i64,
        exploit: Utility,
        elapsed: i64,
        stamped: i64,
    ) {
        let sql = format!(
            "INSERT INTO {t} (epoch, infos, nodes, exploit, elapsed, stamped) VALUES ($1, $2, $3, $4, $5, $6)",
            t = snapshot()
        );
        measure(
            "stage.snapshot",
            self.execute(
                &sql,
                &[
                    &epoch,
                    &infos,
                    &nodes,
                    &(exploit as f32),
                    &elapsed,
                    &stamped,
                ],
            ),
        )
        .await
        .expect("insert snapshot");
    }
}

#[async_trait::async_trait]
impl Stage for Arc<Client> {
    async fn stage(&self) {
        self.as_ref().stage().await
    }

    async fn merge(&self) {
        self.as_ref().merge().await
    }

    async fn stamp(&self, n: usize) {
        self.as_ref().stamp(n).await
    }

    async fn snapshot(
        &self,
        epoch: i64,
        infos: i64,
        nodes: i64,
        exploit: Utility,
        elapsed: i64,
        stamped: i64,
    ) {
        self.as_ref()
            .snapshot(epoch, infos, nodes, exploit, elapsed, stamped)
            .await
    }
}
