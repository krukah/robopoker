use rbp_pg::*;
use std::sync::Arc;
use tokio_postgres::Client;

/// Stage defines bulk upload operations for fast training.
/// Manages staging table lifecycle and batch epoch updates.
#[async_trait::async_trait]
pub trait Stage: Send + Sync {
    async fn stage(&self);
    async fn merge(&self);
    async fn stamp(&self, n: usize);
}

#[async_trait::async_trait]
impl Stage for Client {
    async fn stage(&self) {
        let sql = format!(
            "DROP   TABLE IF EXISTS {t2};
             CREATE UNLOGGED TABLE  {t2} (LIKE {t1});",
            t1 = BLUEPRINT,
            t2 = STAGING
        );
        self.batch_execute(&sql).await.expect("create staging");
    }
    async fn merge(&self) {
        let sql = format!(
            "INSERT INTO   {t1} (past, present, choices, edge, weight, regret, evalue, counts)
             SELECT              past, present, choices, edge, weight, regret, evalue, counts FROM {t2}
             ON CONFLICT  (past, present, choices, edge)
             DO UPDATE SET
                 weight = EXCLUDED.weight,
                 regret = EXCLUDED.regret,
                 evalue = EXCLUDED.evalue,
                 counts = EXCLUDED.counts;
             DROP TABLE    {t2};",
            t1 = BLUEPRINT,
            t2 = STAGING
        );
        self.batch_execute(&sql).await.expect("upsert blueprint");
    }
    async fn stamp(&self, n: usize) {
        let sql = format!(
            "UPDATE {t} SET value = value + $1 WHERE key = 'current'",
            t = EPOCH
        );
        self.execute(&sql, &[&(n as i64)])
            .await
            .expect("update epoch");
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
}
