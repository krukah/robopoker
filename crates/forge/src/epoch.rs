//! Epoch metadata table schema
use std::sync::OnceLock;

/// Newtype wrapper for epoch counter (enables Schema implementation).
pub struct EpochMeta;

impl daybook::Schema for EpochMeta {
    fn name() -> &'static str {
        daybook::epoch()
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let t = daybook::epoch();
        SQL.get_or_init(|| {
            daybook::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {t} (
                key   TEXT PRIMARY KEY,
                value BIGINT NOT NULL
            );
            INSERT INTO {t} (key, value)
            VALUES ('current', 0)
            ON CONFLICT (key) DO NOTHING;"
            ))
        })
    }

    fn indices() -> &'static str {
        ""
    }

    fn copy() -> &'static str {
        unimplemented!()
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| daybook::leaked(format!("UPDATE {} SET value = 0 WHERE key = 'current'", daybook::epoch())))
    }

    fn freeze() -> &'static str {
        unimplemented!()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }
}
