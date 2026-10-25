//! Regime fingerprint — single-row sanity check guarding against silent drift in regime-
//! affecting constants between training runs.
//!
//! [`Fingerprint::audit`] compares the active [`kicker::Regimen`]'s config for the active
//! regime against the value stored in `fingerprint_<regime>_<version>` and hands back the diff.
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

/// Zero-sized type for the fingerprint table schema.
pub struct Fingerprint;

impl crate::Schema for Fingerprint {
    fn name() -> &'static str {
        crate::fingerprint()
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            crate::Pool::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    config TEXT PRIMARY KEY,
                    set_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );",
                crate::fingerprint(),
            ))
        })
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| crate::Pool::leaked(format!("TRUNCATE TABLE {};", crate::fingerprint())))
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
    /// Compare the live game-tree constants against the recorded ones,
    /// returning the diff when they disagree. `Ok(())` also covers the
    /// un-fingerprinted table, which is a first run rather than drift.
    ///
    /// Read-only: recording is [`check`](Self::check)'s job, so a server
    /// auditing the table never claims a blueprint it did not train.
    pub async fn audit(client: &Client) -> Result<(), String> {
        let live = <dyn kicker::Regimen>::active().config();
        match Self::read(client).await {
            Some(stored) if stored != live => Err(stored.diff(&live)),
            _ => Ok(()),
        }
    }

    /// Log the audit at server startup. Never fatal: postflop and every
    /// unaffected node still serve, so taking the site down over drift
    /// would cost more than it saves — but the operator gets the diff, and
    /// the resolution, in the logs instead of guessing at a blank panel.
    pub async fn report(client: &Client) {
        if let Err(diff) = Self::audit(client).await {
            tracing::error!(
                "regime fingerprint mismatch for ({regime}, {version}): `{table}` was trained \
                 with a different game tree shape than this binary computes, so info sets under \
                 any changed action mask will MISS and read as untrained.\n\nDiff (- stored, \
                 + live):\n{diff}\nResolution: retrain (`trainer --regime {regime} --version \
                 {version} --mode reset`, then `--mode fast`).",
                regime = pokerkit::Regime::current(),
                version = pokerkit::Version::current(),
                table = crate::blueprint(),
            );
        }
    }

    /// Verify the live fingerprint against the stored one. First run records
    /// it; mismatch panics with a diff naming the changed constants.
    pub async fn check(client: &Arc<Client>) {
        let regime = pokerkit::Regime::current();
        let live = <dyn kicker::Regimen>::active().config();
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
                version = pokerkit::Version::current(),
                table = crate::blueprint(),
                diff = stored.diff(&live),
            ),
        }
    }

    /// The stored shape, parsed back into terms. The column is TEXT and the
    /// rendering is the wire format, so this is the codec — narrow enough to
    /// live here rather than pull `kicker/sql` in for one row.
    async fn read(client: &Client) -> Option<kicker::Shape> {
        client
            .query_opt(&format!("SELECT config FROM {} LIMIT 1", crate::fingerprint()), &[])
            .await
            .expect("query fingerprint")
            .map(|row| {
                row.get::<_, String>(0)
                    .parse::<kicker::Shape>()
                    .expect("stored fingerprint parses")
            })
    }

    async fn write(client: &Client, config: &kicker::Shape) {
        client
            .batch_execute(<Self as crate::Schema>::truncates())
            .await
            .expect("truncate fingerprint");
        client
            .execute(&format!("INSERT INTO {} (config) VALUES ($1);", crate::fingerprint()), &[&config.to_string()])
            .await
            .expect("insert fingerprint");
    }
}
