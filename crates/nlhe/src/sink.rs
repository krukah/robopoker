//! Database write operations for NLHE-specific types.
//!
//! Requires the `database` feature.
use super::*;
use rbp_database::*;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

/// Sink defines the write interface between NLHE domain types and PostgreSQL.
/// All INSERT/UPDATE queries are consolidated here.
#[async_trait::async_trait]
pub trait Sink: Send + Sync {
    async fn submit(&self, records: Vec<Record>);
    async fn advance(&self);
}

fn upsert_sql() -> &'static str {
    static SQL: OnceLock<&str> = OnceLock::<&str>::new();
    SQL.get_or_init(|| {
        leaked(format!(
            "INSERT INTO {} (past, present, choices, geometry, edge, weight, regret, payoff, visits) \
         VALUES         ($1,   $2,      $3,       $4,       $5,   $6,     $7,     $8,     $9) \
         ON CONFLICT (past, present, choices, geometry, edge) \
         DO UPDATE SET \
             weight = EXCLUDED.weight, \
             regret = EXCLUDED.regret, \
             payoff = EXCLUDED.payoff, \
             visits = EXCLUDED.visits",
            blueprint()
        ))
    })
}
fn advance_sql() -> &'static str {
    static SQL: OnceLock<&str> = OnceLock::<&str>::new();
    SQL.get_or_init(|| {
        leaked(format!(
            "UPDATE {} SET value = value + 1 WHERE key = 'current'",
            epoch()
        ))
    })
}

#[async_trait::async_trait]
impl Sink for Client {
    async fn submit(&self, records: Vec<Record>) {
        for record in records {
            self.execute(
                upsert_sql(),
                &[
                    &i64::from(record.info.subgame()),
                    &i16::from(record.info.bucket()),
                    &i64::from(record.info.choices()),
                    &(record.info.geometry().tag() as i16),
                    &(u64::from(record.edge) as i64),
                    &record.weight,
                    &record.regret,
                    &record.payoff,
                    &(record.visits as i32),
                ],
            )
            .await
            .expect("blueprint upsert");
        }
    }

    async fn advance(&self) {
        self.execute(advance_sql(), &[])
            .await
            .expect("epoch advance");
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
