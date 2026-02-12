use super::*;
use rbp_cards::*;
use rbp_clustering::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_nlhe::*;
use rbp_pg::*;
use const_format::concatcp;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio_postgres::Client;

/// Source defines the read interface between domain types and PostgreSQL.
/// All SELECT queries are consolidated here, decoupling SQL from business logic.
#[async_trait::async_trait]
pub trait Source: Send + Sync {
    async fn memory(&self, info: NlheInfo) -> Memory;
    async fn encode(&self, iso: Isomorphism) -> Abstraction;
    async fn equity(&self, abs: Abstraction) -> Probability;
    async fn metric(&self, street: Street) -> Metric;
    async fn distance(&self, pair: Pair) -> Energy;
    async fn strategy(&self, info: NlheInfo) -> Vec<(Edge, Probability)>;
    async fn histogram(&self, abs: Abstraction) -> Histogram;
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
    async fn metric(&self, street: Street) -> Metric {
        const SQL: &str = concatcp!(
            "SELECT   a1.abs # a2.abs AS tri, ",
                     "m.dx AS dx ",
            "FROM     ", ABSTRACTION, " a1 ",
            "JOIN     ", ABSTRACTION, " a2 ON a1.street = a2.street ",
            "JOIN     ", METRIC,      " m  ON (a1.abs # a2.abs) = m.tri ",
            "WHERE    a1.street = $1 ",
            "AND      a1.abs != a2.abs"
        );
        self.query(SQL, &[&(street as i16)])
            .await
            .expect("metric lookup")
            .iter()
            .map(|row| (row.get::<_, i32>(0), row.get::<_, Energy>(1)))
            .map(|(tri, distance)| (Pair::from(tri), distance))
            .collect::<BTreeMap<Pair, Energy>>()
            .into()
    }
    async fn distance(&self, pair: Pair) -> Energy {
        const SQL: &str = concatcp!(
            "SELECT m.dx ",
            "FROM   ", METRIC, " m ",
            "WHERE  $1 = m.tri"
        );
        self.query_one(SQL, &[&i32::from(pair)])
            .await
            .expect("distance lookup")
            .get::<_, Energy>(0)
    }
    async fn histogram(&self, abs: Abstraction) -> Histogram {
        const SQL: &str = concatcp!(
            "SELECT next, ",
                   "dx ",
            "FROM   ", TRANSITIONS, " ",
            "WHERE  prev = $1"
        );
        let street = abs.street().next();
        let mass = abs.street().n_children() as f32;
        self.query(SQL, &[&i16::from(abs)])
            .await
            .expect("histogram lookup")
            .iter()
            .map(|row| (row.get::<_, i16>(0), row.get::<_, Energy>(1)))
            .map(|(next, dx)| (next, (dx * mass).round() as usize))
            .map(|(next, dx)| (Abstraction::from(next), dx))
            .fold(Histogram::empty(street), |mut h, (next, dx)| {
                h.set(next, dx);
                h
            })
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

    async fn metric(&self, street: Street) -> Metric {
        self.as_ref().metric(street).await
    }

    async fn distance(&self, pair: Pair) -> Energy {
        self.as_ref().distance(pair).await
    }

    async fn histogram(&self, abs: Abstraction) -> Histogram {
        self.as_ref().histogram(abs).await
    }
}
