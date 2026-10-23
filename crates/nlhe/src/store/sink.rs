//! Database write operations for NLHE-specific types.
//!
//! Requires the `database` feature.
use super::*;
use std::sync::Arc;
use tokio_postgres::Client;

/// The write interface between NLHE domain types and PostgreSQL: every
/// INSERT/UPDATE lives here.
#[async_trait::async_trait]
pub trait Sink: Send + Sync {
    async fn submit(&self, records: Vec<Record>);
    async fn advance(&self);
}

#[async_trait::async_trait]
impl Sink for Client {
    async fn submit(&self, records: Vec<Record>) {
        for record in records {
            self.execute(
                Upsert::upsert_sql(),
                &[
                    &record.info().subgame(),
                    &record.info().bucket(),
                    &record.info().choices(),
                    &record.info().field(),
                    &record.edge(),
                    &record.weight(),
                    &record.regret(),
                    &record.payoff(),
                    &(record.visits() as i32),
                ],
            )
            .await
            .expect("blueprint upsert");
        }
    }

    async fn advance(&self) {
        self.execute(Upsert::advance_sql(), &[]).await.expect("epoch advance");
    }
}

#[async_trait::async_trait]
impl Sink for Arc<Client> {
    async fn submit(&self, records: Vec<Record>) {
        self.as_ref().submit(records).await;
    }

    async fn advance(&self) {
        self.as_ref().advance().await;
    }
}
