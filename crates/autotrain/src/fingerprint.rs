//! Regime fingerprint — single-row sanity check guarding against silent
//! drift in regime-affecting constants between training runs.
//!
//! Trainer startup compares [`rbp_core::config_string`] for the active
//! regime against the value stored in `fingerprint_<regime>_<version>`.
//! Mismatch panics with a diff so the operator knows what changed; first
//! run records the live fingerprint. `--mode reset` clears it.
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

/// Zero-sized type for the fingerprint table schema.
pub struct Fingerprint;

impl rbp_database::Schema for Fingerprint {
    fn name() -> &'static str {
        rbp_database::fingerprint()
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            rbp_database::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    config TEXT PRIMARY KEY,
                    set_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );",
                rbp_database::fingerprint(),
            ))
        })
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            rbp_database::leaked(format!("TRUNCATE TABLE {};", rbp_database::fingerprint()))
        })
    }

    fn indices() -> &'static str {
        ""
    }

    fn copy() -> &'static str {
        unimplemented!()
    }

    fn freeze() -> &'static str {
        unimplemented!()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }
}

impl Fingerprint {
    /// Verify the live fingerprint against the stored one. First run records
    /// it; mismatch panics with a diff naming the changed constants.
    pub async fn check(client: &Arc<Client>) {
        let regime = rbp_core::regime();
        let live = rbp_core::config_string(regime);
        match Self::read(client).await {
            None => {
                tracing::info!(%regime, "regime fingerprint absent — recording first run");
                Self::write(client, &live).await;
            }
            Some(stored) if stored == live => {
                tracing::info!(%regime, "regime fingerprint matches");
            }
            Some(stored) => panic!(
                "regime fingerprint mismatch for ({regime}, {version}): the blueprint in \
                 `{table}` was trained with a different game tree shape than this binary \
                 produces.\n\nDiff (- stored, + live):\n{diff}\nResolutions:\n  \
                 - revert the offending constant change, OR\n  \
                 - bump `Version` and regenerate, OR\n  \
                 - run `trainer --regime {regime} --version {version} --mode reset` to \
                 wipe the blueprint and re-fingerprint.",
                version = rbp_core::version(),
                table = rbp_database::blueprint(),
                diff = diff_lines(&stored, &live),
            ),
        }
    }

    async fn read(client: &Client) -> Option<String> {
        client
            .query_opt(
                &format!("SELECT config FROM {} LIMIT 1", rbp_database::fingerprint()),
                &[],
            )
            .await
            .expect("query fingerprint")
            .map(|row| row.get::<_, String>(0))
    }

    async fn write(client: &Client, config: &str) {
        client
            .batch_execute(<Self as rbp_database::Schema>::truncates())
            .await
            .expect("truncate fingerprint");
        client
            .execute(
                &format!(
                    "INSERT INTO {} (config) VALUES ($1);",
                    rbp_database::fingerprint(),
                ),
                &[&config],
            )
            .await
            .expect("insert fingerprint");
    }
}

fn diff_lines(stored: &str, live: &str) -> String {
    let s = stored.split(';').collect::<std::collections::HashSet<_>>();
    let l = live.split(';').collect::<std::collections::HashSet<_>>();
    s.symmetric_difference(&l)
        .map(|p| {
            if s.contains(p) {
                format!("  - {p}")
            } else {
                format!("  + {p}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
