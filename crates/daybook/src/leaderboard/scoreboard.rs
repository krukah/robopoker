//! Cross-fingerprint scoreboard: persisted success metrics per training configuration, so
//! multiple `(regime, version, fingerprint)` tuples can be ranked side by side before
//! committing to a long training run.
use crate::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio_postgres::Client;

/// Persisted success metrics, keyed by fingerprint, for ranking configs.
#[async_trait::async_trait]
pub trait Scoreboard: Send + Sync {
    async fn record_benchmark(&self, score: &Score);
    async fn record_litmus(&self, epoch: i64, pass: i64, fail: i64, skip: i64, error: i64, stamped: i64);
    async fn leaderboard(&self) -> Vec<Standing>;
}

#[async_trait::async_trait]
impl Scoreboard for Client {
    async fn record_benchmark(&self, score: &Score) {
        let regime = pokerkit::Regime::current().to_string();
        let version = pokerkit::Version::current().to_string();
        let print = <dyn kicker::Regimen>::active().digest().to_string();
        let sql = format!(
            "INSERT INTO {t} \
             (regime, version, fingerprint, opponent, variant, epoch, hands, bb, conf, stddev, aivat, aivat_conf, stamped) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            t = benchmark()
        );
        Measure::measure(
            "scoreboard.benchmark",
            self.execute(
                &sql,
                &[
                    &regime,
                    &version,
                    &print,
                    &score.opponent(),
                    &score.variant(),
                    &score.epoch(),
                    &score.hands(),
                    &score.bb(),
                    &score.conf(),
                    &score.stddev(),
                    &score.aivat(),
                    &score.aconf(),
                    &score.stamped(),
                ],
            ),
        )
        .await
        .expect("insert benchmark");
    }

    async fn record_litmus(&self, epoch: i64, pass: i64, fail: i64, skip: i64, error: i64, stamped: i64) {
        let regime = pokerkit::Regime::current().to_string();
        let version = pokerkit::Version::current().to_string();
        let print = <dyn kicker::Regimen>::active().digest().to_string();
        let sql = format!(
            "INSERT INTO {t} (regime, version, fingerprint, epoch, pass, fail, skip, error, stamped) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            t = litmus()
        );
        Measure::measure(
            "scoreboard.litmus",
            self.execute(&sql, &[&regime, &version, &print, &epoch, &pass, &fail, &skip, &error, &stamped]),
        )
        .await
        .expect("insert litmus");
    }

    async fn leaderboard(&self) -> Vec<Standing> {
        let scores = Measure::measure(
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
        let mut ranked = Measure::measure(
            "scoreboard.rank.bench",
            self.query(
                &format!(
                    "SELECT DISTINCT ON (regime, version, opponent, variant) \
                     regime, version, fingerprint, opponent, variant, epoch, hands, bb, conf, aivat, stamped \
                     FROM {t} ORDER BY regime, version, opponent, variant, stamped DESC",
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
                opponent: Some(r.get(3)),
                variant: Some(r.get(4)),
                epoch: r.get(5),
                hands: r.get(6),
                bb: Some(r.get::<_, f64>(7)),
                conf: Some(r.get::<_, f64>(8)),
                aivat: r.get::<_, Option<f64>>(9),
                stamped: r.get(10),
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
                    opponent: None,
                    variant: None,
                    epoch: 0,
                    hands: 0,
                    bb: None,
                    conf: None,
                    aivat: None,
                    litmus: Some(litmus),
                    stamped: 0,
                }),
        );
        ranked.sort_by(|a, b| {
            b.bb()
                .unwrap_or(f64::NEG_INFINITY)
                .partial_cmp(&a.bb().unwrap_or(f64::NEG_INFINITY))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }
}

#[async_trait::async_trait]
impl Scoreboard for Arc<Client> {
    async fn record_benchmark(&self, score: &Score) {
        self.as_ref().record_benchmark(score).await;
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
