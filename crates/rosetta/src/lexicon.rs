//! The database side: sample buckets, write glosses, read them back.
use crate::*;
use daybook::*;
use deuce::*;
use futures::StreamExt;
use kicker::*;
use pokerkit::*;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

/// Reading and writing the `mechinterp` table, and the sampling that feeds it.
///
/// Every read is index-served: the census is one grouped scan of `transitions`
/// against the 782-row `abstraction` table, and each sample is a
/// `(abs, position)` range lookup rather than a scan of the 123M-row
/// `isomorphism` table. Interpreting the whole abstraction costs a few seconds
/// and touches a few tens of thousands of rows.
#[async_trait::async_trait]
pub trait Lexicon {
    /// Buckets sampled at once. The connection pipelines, so this is bounded
    /// by politeness to the database rather than by the client.
    const FANOUT: usize = 16;

    async fn census(&self, street: Street) -> Vec<Census>;
    async fn sample(&self, census: &Census, size: usize) -> Vec<Observation>;
    async fn profiles(&self, street: Street, size: usize) -> Vec<Profile>;
    async fn inscribe(&self, glossary: &Glossary);
    async fn lookup(&self, abs: Abstraction) -> Option<Gloss>;
    async fn glossary(&self, street: Street) -> Glossary;
}

#[async_trait::async_trait]
impl Lexicon for Client {
    /// Population, street share, and the first two moments of the equity
    /// distribution each bucket flows into — the scalar half of a profile,
    /// for every bucket on a street, in one query.
    async fn census(&self, street: Street) -> Vec<Census> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH centroid AS ( \
                 SELECT   t.prev AS abs, \
                          SUM(t.dx * a.equity) AS mu, \
                          SUM(t.dx * a.equity * a.equity) AS m2 \
                 FROM     {t} t \
                 JOIN     {a} a ON a.abs = t.next \
                 GROUP BY t.prev \
                 ) \
                 SELECT   b.abs, \
                          b.population, \
                          (b.population::float8 / NULLIF(SUM(b.population) OVER (), 0))::float4 AS share, \
                          b.equity, \
                          COALESCE(SQRT(GREATEST(c.m2 - c.mu * c.mu, 0)), 0)::float4 AS spread \
                 FROM     {a} b \
                 LEFT     JOIN centroid c ON c.abs = b.abs \
                 WHERE    b.street = $1 \
                 ORDER BY b.abs",
                t = transitions(),
                a = abstraction()
            )
        });
        self.query(sql.as_str(), &[&(street as i16)])
            .await
            .inspect_err(|e| tracing::error!(error = %e, "census"))
            .unwrap_or_default()
            .iter()
            .map(|row| {
                Census::from((
                    row.get::<_, Abstraction>(0),
                    row.get::<_, i32>(1) as usize,
                    row.get::<_, Probability>(2),
                    row.get::<_, Probability>(3),
                    row.get::<_, Probability>(4),
                ))
            })
            .collect()
    }

    /// A uniform sample of a bucket's members, drawn by dense position so the
    /// query rides the `(abs, position)` index and never scans the bucket.
    async fn sample(&self, census: &Census, size: usize) -> Vec<Observation> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql =
            SQL.get_or_init(|| format!("SELECT obs FROM {} WHERE abs = $1 AND position = ANY($2)", isomorphism()));
        self.query(sql.as_str(), &[&census.abs(), &census.positions(size)])
            .await
            .inspect_err(|e| tracing::error!(error = %e, abs = %census.abs(), "sample"))
            .unwrap_or_default()
            .iter()
            .map(|row| row.get::<_, Isomorphism>(0))
            .map(Observation::from)
            .collect()
    }

    /// Every bucket on a street, sampled and tallied.
    async fn profiles(&self, street: Street, size: usize) -> Vec<Profile> {
        futures::stream::iter(self.census(street).await)
            .map(|census| async move { Profile::from((census, self.sample(&census, size).await)) })
            .buffered(Self::FANOUT)
            .collect::<Vec<Profile>>()
            .await
    }

    /// Upserts a whole street in one statement, keyed on the bucket id, so a
    /// re-run rewrites names in place instead of accumulating stale ones.
    async fn inscribe(&self, glossary: &Glossary) {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "INSERT INTO {} (abs, name, description) \
                 SELECT * FROM UNNEST($1::SMALLINT[], $2::TEXT[], $3::TEXT[]) \
                 ON CONFLICT (abs) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description",
                mechinterp()
            )
        });
        self.ensure::<Gloss>().await;
        measure(
            "mechinterp.inscribe",
            self.execute(
                sql.as_str(),
                &[
                    &glossary.glosses().iter().map(Gloss::abs).collect::<Vec<Abstraction>>(),
                    &glossary
                        .glosses()
                        .iter()
                        .map(|g| String::from(g.name()))
                        .collect::<Vec<String>>(),
                    &glossary
                        .glosses()
                        .iter()
                        .map(|g| String::from(g.description()))
                        .collect::<Vec<String>>(),
                ],
            ),
        )
        .await
        .inspect_err(|e| tracing::error!(error = %e, "inscribe"))
        .ok();
    }

    /// The gloss for one bucket, if it has been interpreted yet.
    async fn lookup(&self, abs: Abstraction) -> Option<Gloss> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT abs, name, description FROM {} WHERE abs = $1", mechinterp()));
        self.query_opt(sql.as_str(), &[&abs])
            .await
            .ok()
            .flatten()
            .map(|row| Gloss::from((row.get::<_, Abstraction>(0), row.get::<_, String>(1), row.get::<_, String>(2))))
    }

    /// Every gloss on a street, joined through the abstraction table that
    /// owns the street column.
    async fn glossary(&self, street: Street) -> Glossary {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT   m.abs, m.name, m.description \
                 FROM     {m} m \
                 JOIN     {a} a ON a.abs = m.abs \
                 WHERE    a.street = $1 \
                 ORDER BY m.abs",
                m = mechinterp(),
                a = abstraction()
            )
        });
        Glossary::from(
            self.query(sql.as_str(), &[&(street as i16)])
                .await
                .inspect_err(|e| tracing::error!(error = %e, "glossary"))
                .unwrap_or_default()
                .iter()
                .map(|row| {
                    Gloss::from((row.get::<_, Abstraction>(0), row.get::<_, String>(1), row.get::<_, String>(2)))
                })
                .collect::<Vec<Gloss>>(),
        )
    }
}

#[async_trait::async_trait]
impl Lexicon for Arc<Client> {
    async fn census(&self, street: Street) -> Vec<Census> {
        self.as_ref().census(street).await
    }

    async fn sample(&self, census: &Census, size: usize) -> Vec<Observation> {
        self.as_ref().sample(census, size).await
    }

    async fn profiles(&self, street: Street, size: usize) -> Vec<Profile> {
        self.as_ref().profiles(street, size).await
    }

    async fn inscribe(&self, glossary: &Glossary) {
        self.as_ref().inscribe(glossary).await;
    }

    async fn lookup(&self, abs: Abstraction) -> Option<Gloss> {
        self.as_ref().lookup(abs).await
    }

    async fn glossary(&self, street: Street) -> Glossary {
        self.as_ref().glossary(street).await
    }
}
