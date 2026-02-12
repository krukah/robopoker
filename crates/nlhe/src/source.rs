//! Database read operations for NLHE-specific types.
//!
//! Requires the `database` feature.
use super::*;
use rbp_cards::*;
use rbp_core::*;
use rbp_database::*;
use rbp_gameplay::*;
use const_format::concatcp;
use std::sync::Arc;
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
        const SQL: &str = concatcp!(
            "SELECT abs ",
            "FROM   ", ISOMORPHISM, " ",
            "WHERE  obs = $1"
        );
        self.query_one(SQL, &[&i64::from(iso)])
            .await
            .expect("isomorphism lookup")
            .get::<_, i16>(0)
            .into()
    }
    async fn memory(&self, info: NlheInfo) -> Memory {
        const SQL: &str = concatcp!(
            "SELECT edge, ",
                   "weight, ",
                   "regret, ",
                   "evalue, ",
                   "counts ",
            "FROM   ", BLUEPRINT, " ",
            "WHERE  past    = $1 ",
            "AND    present = $2 ",
            "AND    choices = $3"
        );
        let data = self
            .query(SQL, &[&i64::from(info.subgame()), &i16::from(info.bucket()), &i64::from(info.choices())])
            .await
            .expect("memory lookup")
            .into_iter()
            .map(|row| {
                let edge = Edge::from(row.get::<_, i64>(0) as u64);
                let weight = row.get::<_, f32>(1);
                let regret = row.get::<_, f32>(2);
                let evalue = row.get::<_, f32>(3);
                let counts = row.get::<_, i32>(4) as u32;
                (edge, weight, regret, evalue, counts)
            })
            .collect();
        Memory::new(info, data)
    }
    async fn strategy(&self, info: NlheInfo) -> Vec<(Edge, Probability)> {
        const SQL: &str = concatcp!(
            "SELECT edge, ",
                   "weight ",
            "FROM   ", BLUEPRINT, " ",
            "WHERE  past    = $1 ",
            "AND    present = $2 ",
            "AND    choices = $3"
        );
        self.query(SQL, &[&i64::from(info.subgame()), &i16::from(info.bucket()), &i64::from(info.choices())])
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
        const SQL: &str = concatcp!(
            "SELECT equity ",
            "FROM   ", ABSTRACTION, " ",
            "WHERE  abs = $1"
        );
        self.query_one(SQL, &[&i16::from(abs)])
            .await
            .expect("equity lookup")
            .get::<_, f32>(0)
    }
    async fn population(&self, abs: Abstraction) -> usize {
        const SQL: &str = concatcp!(
            "SELECT population ",
            "FROM   ", ABSTRACTION, " ",
            "WHERE  abs = $1"
        );
        self.query_one(SQL, &[&i16::from(abs)])
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
