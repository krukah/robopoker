//! Epoch metadata table schema

/// Newtype wrapper for epoch counter (enables Schema implementation).
pub struct EpochMeta;

impl rbp_database::Schema for EpochMeta {
    fn name() -> &'static str {
        rbp_database::EPOCH
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            rbp_database::EPOCH,
            " (
                key   TEXT PRIMARY KEY,
                value BIGINT NOT NULL
            );
            INSERT INTO ",
            rbp_database::EPOCH,
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
        const_format::concatcp!(
            "UPDATE ",
            rbp_database::EPOCH,
            " SET value = 0 WHERE key = 'current'"
        )
    }
    fn freeze() -> &'static str {
        unimplemented!()
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }
}
