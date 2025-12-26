use crate::save::*;
use crate::workers::Record;
use std::sync::Arc;
use tokio_postgres::Client;

/// Sink defines the write interface between domain types and PostgreSQL.
/// All INSERT/UPDATE queries are consolidated here.
#[async_trait::async_trait]
pub trait Sink: Send + Sync {
    async fn submit(&self, records: Vec<Record>);
    async fn advance(&self);
}

#[async_trait::async_trait]
impl Sink for Client {
    async fn submit(&self, records: Vec<Record>) {
        #[rustfmt::skip]
        const SQL: &str = const_format::concatcp!(
            "INSERT INTO ", BLUEPRINT, " (past, present, future, edge, policy, regret) ",
            "VALUES                      ($1,   $2,      $3,     $4,   $5,     $6) ",
            "ON CONFLICT (past, present, future, edge) ",
            "DO UPDATE SET ",
                "policy = EXCLUDED.policy, ",
                "regret = EXCLUDED.regret"
        );
        for record in records {
            self.execute(
                SQL,
                &[
                    &i64::from(*record.info.history()),
                    &i64::from(*record.info.present()),
                    &i64::from(*record.info.choices()),
                    &(u64::from(record.edge) as i64),
                    &record.policy,
                    &record.regret,
                ],
            )
            .await
            .expect("blueprint upsert");
        }
    }

    async fn advance(&self) {
        #[rustfmt::skip]
        const SQL: &str = const_format::concatcp!(
            "UPDATE ", EPOCH, " ",
            "SET    value = value + 1 ",
            "WHERE  key = 'current'"
        );
        self.execute(SQL, &[]).await.expect("epoch advance");
    }
}

#[async_trait::async_trait]
impl Sink for Arc<Client> {
    async fn submit(&self, records: Vec<Record>) {
        self.as_ref().submit(records).await
    }
    async fn advance(&self) {
        self.as_ref().advance().await
    }
}
