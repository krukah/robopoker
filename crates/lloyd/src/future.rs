use crate::*;
use kicker::*;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Transition model mapping abstractions to next-street histograms.
///
/// Stores the learned cluster centroids: for each abstraction bucket,
/// what is the expected distribution over next-street buckets? This
/// enables propagating beliefs through the abstraction hierarchy.
///
/// # Database
///
/// Persisted to the `transitions` table for use in real-time inference.
#[derive(Default)]
pub struct Future(BTreeMap<Abstraction, Histogram>);

impl From<BTreeMap<Abstraction, Histogram>> for Future {
    fn from(map: BTreeMap<Abstraction, Histogram>) -> Self {
        Self(map)
    }
}

impl From<Future> for BTreeMap<Abstraction, Histogram> {
    fn from(future: Future) -> Self {
        future.0
    }
}

#[cfg(feature = "server")]
impl ledger::Schema for Future {
    fn name() -> &'static str {
        ledger::transitions()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT2,   // prev (source abstraction)
            tokio_postgres::types::Type::INT2,   // next (target abstraction)
            tokio_postgres::types::Type::FLOAT4, // dx (transition probability)
        ]
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            ledger::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                prev SMALLINT NOT NULL,
                next SMALLINT NOT NULL,
                dx   REAL     NOT NULL
            );",
                ledger::transitions()
            ))
        })
    }

    fn indices() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let t = ledger::transitions();
        SQL.get_or_init(|| {
            ledger::leaked(format!(
                "CREATE INDEX IF NOT EXISTS idx_{t}_prev ON {t} (prev);
             CREATE INDEX IF NOT EXISTS idx_{t}_next ON {t} (next);
             CREATE INDEX IF NOT EXISTS idx_{t}_dx ON {t} (dx);
             CREATE INDEX IF NOT EXISTS idx_{t}_next_dx ON {t} (next, dx);
             CREATE INDEX IF NOT EXISTS idx_{t}_next_prev ON {t} (next, prev);
             CREATE INDEX IF NOT EXISTS idx_{t}_prev_dx ON {t} (prev, dx);
             CREATE INDEX IF NOT EXISTS idx_{t}_prev_next ON {t} (prev, next);"
            ))
        })
    }

    fn copy() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| ledger::leaked(format!("COPY {} (prev, next, dx) FROM STDIN BINARY", ledger::transitions())))
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| ledger::leaked(format!("TRUNCATE TABLE {};", ledger::transitions())))
    }

    fn freeze() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let t = ledger::transitions();
        SQL.get_or_init(|| {
            ledger::leaked(format!(
                "ALTER TABLE {t} SET (fillfactor = 100);
             ALTER TABLE {t} SET (autovacuum_enabled = false);"
            ))
        })
    }
}

#[cfg(feature = "server")]
#[async_trait::async_trait]
impl ledger::Streamable for Future {
    type Row = (i16, i16, f32);

    fn rows(self) -> impl Iterator<Item = Self::Row> + Send {
        self.0.into_iter().flat_map(|(abs, hist)| {
            let prev = i16::from(abs);
            hist.distribution()
                .into_iter()
                .map(move |(abs, dx)| (prev, i16::from(abs), dx))
        })
    }
}
