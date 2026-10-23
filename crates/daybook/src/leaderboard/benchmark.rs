use crate::*;
use std::sync::OnceLock;

/// External-opponent bb/100 scoreboard table.
pub struct Benchmark;
impl Schema for Benchmark {
    fn name() -> &'static str {
        benchmark()
    }

    fn copy() -> &'static str {
        unimplemented!()
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            Pool::leaked(format!(
                "CREATE TABLE IF NOT EXISTS {0} (
                id          BIGSERIAL PRIMARY KEY,
                regime      TEXT             NOT NULL,
                version     TEXT             NOT NULL,
                fingerprint TEXT             NOT NULL,
                opponent    TEXT             NOT NULL DEFAULT 'slumbot',
                variant     TEXT             NOT NULL,
                epoch       BIGINT           NOT NULL,
                hands       BIGINT           NOT NULL,
                bb          DOUBLE PRECISION NOT NULL,
                conf        DOUBLE PRECISION NOT NULL,
                stddev      DOUBLE PRECISION NOT NULL,
                aivat       DOUBLE PRECISION,
                aivat_conf  DOUBLE PRECISION,
                stamped     BIGINT           NOT NULL
            );
            ALTER TABLE {0} ADD COLUMN IF NOT EXISTS opponent   TEXT NOT NULL DEFAULT 'slumbot';
            ALTER TABLE {0} ADD COLUMN IF NOT EXISTS aivat      DOUBLE PRECISION;
            ALTER TABLE {0} ADD COLUMN IF NOT EXISTS aivat_conf DOUBLE PRECISION;",
                benchmark()
            ))
        })
    }

    fn indices() -> &'static str {
        ""
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| Pool::leaked(format!("TRUNCATE TABLE {};", benchmark())))
    }

    fn freeze() -> &'static str {
        unimplemented!()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!()
    }
}
