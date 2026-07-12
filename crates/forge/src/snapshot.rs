//! Snapshot table schema for periodic training statistics.
use std::sync::OnceLock;

/// Zero-sized type for snapshot table schema.
pub struct Snapshot;

impl daybook::Schema for Snapshot {
    fn name() -> &'static str {
        daybook::snapshot()
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            daybook::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                id      BIGSERIAL PRIMARY KEY,
                epoch   BIGINT NOT NULL,
                infos   BIGINT NOT NULL,
                nodes   BIGINT NOT NULL,
                exploit REAL,
                elapsed BIGINT NOT NULL,
                stamped BIGINT NOT NULL
            );",
                daybook::snapshot()
            ))
        })
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| daybook::leaked(format!("TRUNCATE TABLE {};", daybook::snapshot())))
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
