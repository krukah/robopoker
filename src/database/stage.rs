use crate::save::*;
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
             CREATE UNLOGGED TABLE  {t2} (LIKE {t1} INCLUDING ALL);",
            t1 = BLUEPRINT,
            t2 = STAGING
        );
        self.batch_execute(&sql).await.expect("create staging");
    }
    async fn merge(&self) {
        let sql = format!(
            "INSERT INTO   {t1} (past, present, future, edge, policy, regret)
             SELECT              past, present, future, edge, policy, regret FROM {t2}
             ON CONFLICT  (past, present, future, edge)
             DO UPDATE SET
                 policy = EXCLUDED.policy,
                 regret = EXCLUDED.regret;
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
