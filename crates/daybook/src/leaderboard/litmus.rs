use crate::*;
use std::sync::OnceLock;

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
            Pool::leaked(format!(
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
        SQL.get_or_init(|| Pool::leaked(format!("TRUNCATE TABLE {};", litmus())))
    }

    fn freeze() -> &'static str {
        unimplemented!()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }
}
