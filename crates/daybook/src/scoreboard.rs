//! Cross-fingerprint scoreboard: persisted success metrics per training
//! configuration, so multiple `(regime, version, fingerprint)` tuples can be
//! ranked side by side before committing to a long training run.
//!
//! Two append-only tables, both SHARED (`table!`, no regime/version suffix)
//! and self-describing via explicit `regime` / `version` / `fingerprint`
//! columns:
//!
//! - [`Benchmark`] — Slumbot bb/100 per `(fingerprint, variant)`.
//! - [`Litmus`] — structural pass/fail counts per fingerprint.
//!
//! Both are `DOUBLE PRECISION` (f64) for the score columns — the `float4`
//! deserializer panic (see CLAUDE.md) only bites when SQL returns `float8`
//! into a Rust `f32`, so keeping everything `f64` sidesteps it entirely.
use crate::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

/// Slumbot bb/100 scoreboard table.
pub struct Benchmark;

impl Schema for Benchmark {
    fn name() -> &'static str {
        benchmark()
    }

    fn copy() -> &'static str {
        unimplemented!()
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                id          BIGSERIAL PRIMARY KEY,
                regime      TEXT             NOT NULL,
                version     TEXT             NOT NULL,
                fingerprint TEXT             NOT NULL,
                variant     TEXT             NOT NULL,
                epoch       BIGINT           NOT NULL,
                hands       BIGINT           NOT NULL,
                bb          DOUBLE PRECISION NOT NULL,
                conf        DOUBLE PRECISION NOT NULL,
                stddev      DOUBLE PRECISION NOT NULL,
                stamped     BIGINT           NOT NULL
            );",
                benchmark()
            ))
        })
    }

    fn indices() -> &'static str {
        ""
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| leaked(format!("TRUNCATE TABLE {};", benchmark())))
    }

    fn freeze() -> &'static str {
        unimplemented!()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }
}

/// Litmus structural pass/fail scoreboard table.
pub struct Litmus;

impl Schema for Litmus {
    fn name() -> &'static str {
        litmus()
    }

    fn copy() -> &'static str {
        unimplemented!()
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                id          BIGSERIAL PRIMARY KEY,
                regime      TEXT   NOT NULL,
                version     TEXT   NOT NULL,
                fingerprint TEXT   NOT NULL,
                epoch       BIGINT NOT NULL,
                pass        BIGINT NOT NULL,
                fail        BIGINT NOT NULL,
                skip        BIGINT NOT NULL,
                error       BIGINT NOT NULL,
                stamped     BIGINT NOT NULL
            );",
                litmus()
            ))
        })
    }

    fn indices() -> &'static str {
        ""
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| leaked(format!("TRUNCATE TABLE {};", litmus())))
    }

    fn freeze() -> &'static str {
        unimplemented!()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }
}

/// Litmus pass/fail/skip/error counts for one fingerprint.
#[derive(Clone, serde::Serialize)]
pub struct Litscore {
    pass: i64,
    fail: i64,
    skip: i64,
    error: i64,
}

impl Litscore {
    /// Fraction of decisive (pass + fail) cases that passed. Skips and
    /// errors are excluded — they measure catalog coverage, not quality.
    pub fn rate(&self) -> f64 {
        let decisive = self.pass + self.fail;
        if decisive == 0 { 0.0 } else { self.pass as f64 / decisive as f64 }
    }
}

/// One ranked row: the latest Slumbot benchmark for a
/// `(regime, version, variant)` fingerprint, plus that fingerprint's latest
/// litmus score. A fingerprint that has litmus but no benchmark appears with
/// `variant = None` and `bb = None` so it is never silently dropped.
#[derive(serde::Serialize)]
pub struct Standing {
    regime: String,
    version: String,
    fingerprint: String,
    variant: Option<String>,
    epoch: i64,
    hands: i64,
    bb: Option<f64>,
    conf: Option<f64>,
    litmus: Option<Litscore>,
    stamped: i64,
}

/// Persisted success metrics, keyed by fingerprint, for ranking configs.
#[async_trait::async_trait]
pub trait Scoreboard: Send + Sync {
    async fn record_benchmark(
        &self,
        variant: &str,
        epoch: i64,
        hands: i64,
        bb: f64,
        conf: f64,
        stddev: f64,
        stamped: i64,
    );
    async fn record_litmus(&self, epoch: i64, pass: i64, fail: i64, skip: i64, error: i64, stamped: i64);
    async fn leaderboard(&self) -> Vec<Standing>;
}

#[async_trait::async_trait]
impl Scoreboard for Client {
    async fn record_benchmark(
        &self,
        variant: &str,
        epoch: i64,
        hands: i64,
        bb: f64,
        conf: f64,
        stddev: f64,
        stamped: i64,
    ) {
        let regime = pokerkit::regime().to_string();
        let version = pokerkit::version().to_string();
        let print = pokerkit::fingerprint(pokerkit::regime());
        let sql = format!(
            "INSERT INTO {t} (regime, version, fingerprint, variant, epoch, hands, bb, conf, stddev, stamped) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            t = benchmark()
        );
        measure(
            "scoreboard.benchmark",
            self.execute(
                &sql,
                &[
                    &regime, &version, &print, &variant, &epoch, &hands, &bb, &conf, &stddev, &stamped,
                ],
            ),
        )
        .await
        .expect("insert benchmark");
    }

    async fn record_litmus(&self, epoch: i64, pass: i64, fail: i64, skip: i64, error: i64, stamped: i64) {
        let regime = pokerkit::regime().to_string();
        let version = pokerkit::version().to_string();
        let print = pokerkit::fingerprint(pokerkit::regime());
        let sql = format!(
            "INSERT INTO {t} (regime, version, fingerprint, epoch, pass, fail, skip, error, stamped) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            t = litmus()
        );
        measure(
            "scoreboard.litmus",
            self.execute(&sql, &[&regime, &version, &print, &epoch, &pass, &fail, &skip, &error, &stamped]),
        )
        .await
        .expect("insert litmus");
    }

    async fn leaderboard(&self) -> Vec<Standing> {
        let scores = measure(
            "scoreboard.rank.litmus",
            self.query(
                &format!(
                    "SELECT DISTINCT ON (regime, version) regime, version, pass, fail, skip, error \
                     FROM {t} ORDER BY regime, version, stamped DESC",
                    t = litmus()
                ),
                &[],
            ),
        )
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| {
            (
                (r.get::<_, String>(0), r.get::<_, String>(1)),
                Litscore {
                    pass: r.get(2),
                    fail: r.get(3),
                    skip: r.get(4),
                    error: r.get(5),
                },
            )
        })
        .collect::<HashMap<(String, String), Litscore>>();
        let mut seen = HashSet::<(String, String)>::new();
        let mut ranked = measure(
            "scoreboard.rank.bench",
            self.query(
                &format!(
                    "SELECT DISTINCT ON (regime, version, variant) \
                     regime, version, fingerprint, variant, epoch, hands, bb, conf, stamped \
                     FROM {t} ORDER BY regime, version, variant, stamped DESC",
                    t = benchmark()
                ),
                &[],
            ),
        )
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| {
            let regime = r.get::<_, String>(0);
            let version = r.get::<_, String>(1);
            seen.insert((regime.clone(), version.clone()));
            Standing {
                fingerprint: r.get(2),
                variant: Some(r.get(3)),
                epoch: r.get(4),
                hands: r.get(5),
                bb: Some(r.get::<_, f64>(6)),
                conf: Some(r.get::<_, f64>(7)),
                stamped: r.get(8),
                litmus: scores.get(&(regime.clone(), version.clone())).cloned(),
                regime,
                version,
            }
        })
        .collect::<Vec<Standing>>();
        ranked.extend(
            scores
                .into_iter()
                .filter(|(key, _)| !seen.contains(key))
                .map(|((regime, version), litmus)| Standing {
                    regime,
                    version,
                    fingerprint: String::new(),
                    variant: None,
                    epoch: 0,
                    hands: 0,
                    bb: None,
                    conf: None,
                    litmus: Some(litmus),
                    stamped: 0,
                }),
        );
        ranked.sort_by(|a, b| {
            b.bb.unwrap_or(f64::NEG_INFINITY)
                .partial_cmp(&a.bb.unwrap_or(f64::NEG_INFINITY))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }
}

#[async_trait::async_trait]
impl Scoreboard for Arc<Client> {
    async fn record_benchmark(
        &self,
        variant: &str,
        epoch: i64,
        hands: i64,
        bb: f64,
        conf: f64,
        stddev: f64,
        stamped: i64,
    ) {
        self.as_ref()
            .record_benchmark(variant, epoch, hands, bb, conf, stddev, stamped)
            .await;
    }

    async fn record_litmus(&self, epoch: i64, pass: i64, fail: i64, skip: i64, error: i64, stamped: i64) {
        self.as_ref()
            .record_litmus(epoch, pass, fail, skip, error, stamped)
            .await;
    }

    async fn leaderboard(&self) -> Vec<Standing> {
        self.as_ref().leaderboard().await
    }
}

/// Render the ranked leaderboard as an aligned text table: `(regime, version,
/// variant)` ordered by bb/100 descending, with litmus pass/total alongside.
pub fn render(standings: &[Standing]) -> String {
    let header = format!(
        "    {:<9} {:<4} {:<12} {:>11} {:>7} {:>8} {:>6} {:>8}",
        "regime", "ver", "variant", "epoch", "hands", "bb/100", "±95", "litmus"
    );
    let rule = "─".repeat(header.chars().count());
    let rows = standings
        .iter()
        .enumerate()
        .map(|(i, s)| {
            format!(
                "{:>2}. {:<9} {:<4} {:<12} {:>11} {:>7} {:>8} {:>6} {:>8}",
                i + 1,
                s.regime,
                s.version,
                s.variant.as_deref().unwrap_or("—"),
                s.epoch,
                s.hands,
                s.bb.map_or_else(|| "—".to_string(), |b| format!("{b:+.1}")),
                s.conf.map_or_else(|| "—".to_string(), |c| format!("{c:.1}")),
                s.litmus
                    .as_ref()
                    .map_or_else(|| "—".to_string(), |l| format!("{}/{}", l.pass, l.pass + l.fail)),
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    format!("{header}\n{rule}\n{rows}")
}
