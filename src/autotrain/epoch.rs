//! Epoch metadata table schema

/// Zero-size type for epoch metadata table.
/// Implements Schema for schema parity with other tables.
pub struct Epoch;

#[cfg(feature = "database")]
impl crate::save::Schema for Epoch {
    fn name() -> &'static str {
        crate::save::EPOCH
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            crate::save::EPOCH,
            " (
                key   TEXT PRIMARY KEY,
                value BIGINT NOT NULL
            );
            INSERT INTO ",
            crate::save::EPOCH,
            " (key, value)
            VALUES ('current', 0)
            ON CONFLICT (key) DO NOTHING;"
        )
    }
    fn indices() -> &'static str {
        unimplemented!()
    }
    fn copy() -> &'static str {
        unimplemented!()
    }
    fn truncates() -> &'static str {
        unimplemented!()
    }
    fn freeze() -> &'static str {
        unimplemented!()
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }
}
