use super::*;
use rbp_pg::*;
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
            "INSERT INTO ", BLUEPRINT, " (past, present, choices, edge, weight, regret, evalue, counts) ",
            "VALUES                      ($1,   $2,      $3,      $4,   $5,     $6,     $7,     $8) ",
            "ON CONFLICT (past, present, choices, edge) ",
            "DO UPDATE SET ",
                "weight = EXCLUDED.weight, ",
                "regret = EXCLUDED.regret, ",
                "evalue = EXCLUDED.evalue, ",
                "counts = EXCLUDED.counts"
        );
        for record in records {
            self.execute(
                SQL,
                &[
                    &i64::from(record.info.subgame()),
                    &i16::from(record.info.bucket()),
                    &i64::from(record.info.choices()),
                    &(u64::from(record.edge) as i64),
                    &record.weight,
                    &record.regret,
                    &record.evalue,
                    &(record.counts as i32),
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
