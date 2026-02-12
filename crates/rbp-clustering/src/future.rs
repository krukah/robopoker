use crate::*;
use rbp_gameplay::*;
use std::collections::BTreeMap;

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

#[cfg(feature = "database")]
impl rbp_pg::Schema for Future {
    fn name() -> &'static str {
        rbp_pg::TRANSITIONS
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::INT2,   // prev (source abstraction)
            tokio_postgres::types::Type::INT2,   // next (target abstraction)
            tokio_postgres::types::Type::FLOAT4, // dx (transition probability)
        ]
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            rbp_pg::TRANSITIONS,
            " (
                prev SMALLINT NOT NULL,
                next SMALLINT NOT NULL,
                dx   REAL     NOT NULL
            );"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "CREATE INDEX IF NOT EXISTS idx_transitions_prev ON ",
            rbp_pg::TRANSITIONS,
            " (prev);
             CREATE INDEX IF NOT EXISTS idx_transitions_next ON ",
            rbp_pg::TRANSITIONS,
            " (next);"
        )
    }
    fn copy() -> &'static str {
        const_format::concatcp!(
            "COPY ",
            rbp_pg::TRANSITIONS,
            " (prev, next, dx) FROM STDIN BINARY"
        )
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", rbp_pg::TRANSITIONS, ";")
    }
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            rbp_pg::TRANSITIONS,
            " SET (fillfactor = 100);
             ALTER TABLE ",
            rbp_pg::TRANSITIONS,
            " SET (autovacuum_enabled = false);"
        )
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl rbp_pg::Streamable for Future {
    type Row = (i16, i16, f32);
    fn rows(self) -> impl Iterator<Item = Self::Row> + Send {
        self.0
            .into_iter()
            .flat_map(|(abs, hist)| {
                let prev = i16::from(abs);
                hist.distribution()
                    .into_iter()
                    .map(move |(abs, dx)| (prev, i16::from(abs), dx))
            })
    }
}
