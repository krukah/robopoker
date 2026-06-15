//! Database read operations for NLHE-specific types.
//!
//! Requires the `database` feature.
use super::*;
use deuce::*;
use kicker::*;
use ledger::*;
use pokerkit::*;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

/// Source defines the read interface between NLHE domain types and PostgreSQL.
/// All SELECT queries are consolidated here, decoupling SQL from business logic.
#[async_trait::async_trait]
pub trait Source: Send + Sync {
    async fn memory(&self, info: NlheInfo) -> Memory;
    async fn encode(&self, iso: Isomorphism) -> Abstraction;
    async fn equity(&self, abs: Abstraction) -> Probability;
    async fn strategy(&self, info: NlheInfo) -> Vec<(Edge, Probability)>;
    async fn population(&self, abs: Abstraction) -> usize;
}

#[rustfmt::skip]
#[async_trait::async_trait]
impl Source for Client {
    async fn encode(&self, iso: Isomorphism) -> Abstraction {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT abs FROM {} WHERE obs = $1",
            isomorphism()
        ));
        self.query_one(sql.as_str(), &[&i64::from(iso)])
            .await
            .expect("isomorphism lookup")
            .get::<_, i16>(0)
            .into()
    }

    async fn memory(&self, info: NlheInfo) -> Memory {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT edge, weight, regret, payoff, visits \
             FROM   {} \
             WHERE  past    = $1 \
             AND    present = $2 \
             AND    choices = $3",
            blueprint()
        ));
        let data = self
            .query(sql.as_str(), &[&i64::from(info.subgame()), &i16::from(info.bucket()), &i64::from(info.choices())])
            .await
            .expect("memory lookup")
            .into_iter()
            .map(|row| {
                let edge = Edge::from(row.get::<_, i64>(0) as u64);
                let weight = row.get::<_, f32>(1);
                let regret = row.get::<_, f32>(2);
                let payoff = row.get::<_, f32>(3);
                let visits = row.get::<_, i32>(4) as u32;
                (edge, weight, regret, payoff, visits)
            })
            .collect();
        Memory::new(info, data)
    }

    async fn strategy(&self, info: NlheInfo) -> Vec<(Edge, Probability)> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT edge, weight \
             FROM   {} \
             WHERE  past    = $1 \
             AND    present = $2 \
             AND    choices = $3",
            blueprint()
        ));
        self.query(sql.as_str(), &[&i64::from(info.subgame()), &i16::from(info.bucket()), &i64::from(info.choices())])
            .await
            .expect("strategy lookup")
            .into_iter()
            .map(|row| {
                let edge = Edge::from(row.get::<_, i64>(0) as u64);
                let weight = row.get::<_, f32>(1);
                (edge, weight)
            })
            .collect()
    }

    async fn equity(&self, abs: Abstraction) -> Probability {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT equity FROM {} WHERE abs = $1",
            abstraction()
        ));
        self.query_one(sql.as_str(), &[&i16::from(abs)])
            .await
            .expect("equity lookup")
            .get::<_, f32>(0)
    }

    async fn population(&self, abs: Abstraction) -> usize {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT population FROM {} WHERE abs = $1",
            abstraction()
        ));
        self.query_one(sql.as_str(), &[&i16::from(abs)])
            .await
            .expect("population lookup")
            .get::<_, i32>(0) as usize
    }
}

#[async_trait::async_trait]
impl Source for Arc<Client> {
    async fn encode(&self, iso: Isomorphism) -> Abstraction {
        self.as_ref().encode(iso).await
    }

    async fn memory(&self, info: NlheInfo) -> Memory {
        self.as_ref().memory(info).await
    }

    async fn strategy(&self, info: NlheInfo) -> Vec<(Edge, Probability)> {
        self.as_ref().strategy(info).await
    }

    async fn equity(&self, abs: Abstraction) -> Probability {
        self.as_ref().equity(abs).await
    }

    async fn population(&self, abs: Abstraction) -> usize {
        self.as_ref().population(abs).await
    }
}
