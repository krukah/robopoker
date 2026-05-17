//! Database schema implementations for domain types.
//!
//! Implements Schema/Derive traits directly on types from other crates.
//! This is possible because Schema/Derive are local to this crate.
use super::*;
use rbp_cards::*;
use rbp_gameplay::*;
use std::sync::OnceLock;

impl Schema for Street {
    fn name() -> &'static str {
        street()
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                street     SMALLINT,
                nobs       INTEGER,
                nabs       INTEGER
            );
            TRUNCATE TABLE {};
            CREATE OR REPLACE FUNCTION get_nabs(s SMALLINT) RETURNS INTEGER AS
            $$ BEGIN RETURN (SELECT COUNT(*) FROM {} a WHERE a.street = s); END; $$
            LANGUAGE plpgsql;",
                street(),
                street(),
                abstraction()
            ))
        })
    }

    fn indices() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            leaked(format!(
                "CREATE INDEX IF NOT EXISTS idx_{0}_st ON {0} (street);",
                street()
            ))
        })
    }

    fn copy() -> &'static str {
        unimplemented!("Street is derived, not loaded from files")
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| leaked(format!("TRUNCATE TABLE {};", street())))
    }

    fn freeze() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            leaked(format!(
                "ALTER TABLE {} SET (fillfactor = 100);
            ALTER TABLE {} SET (autovacuum_enabled = false);",
                street(),
                street()
            ))
        })
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!("Street is derived, not loaded from files")
    }
}

impl Derive for Street {
    fn exhaust() -> Vec<Self> {
        Street::all().iter().rev().copied().collect()
    }

    fn inserts(&self) -> String {
        let s = *self as i16;
        let n = self.n_isomorphisms() as i32;
        format!(
            "INSERT INTO {} (street, nobs, nabs) VALUES ({}, {}, get_nabs({}::SMALLINT));",
            street(),
            s,
            n,
            s
        )
    }
}

impl Schema for Abstraction {
    fn name() -> &'static str {
        abstraction()
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                abs         SMALLINT,
                street      SMALLINT,
                population  INTEGER,
                equity      REAL
            );
            TRUNCATE TABLE {};
            CREATE OR REPLACE FUNCTION get_population(xxx SMALLINT) RETURNS INTEGER AS
            $$ BEGIN RETURN (SELECT COUNT(*) FROM {} e WHERE e.abs = xxx); END; $$
            LANGUAGE plpgsql;
            CREATE OR REPLACE FUNCTION get_street_abs(abs SMALLINT) RETURNS SMALLINT AS
            $$ BEGIN RETURN ((abs >> 8) & 255)::SMALLINT; END; $$
            LANGUAGE plpgsql;
            CREATE OR REPLACE FUNCTION get_equity(parent SMALLINT) RETURNS REAL AS
            $$ BEGIN RETURN CASE WHEN get_street_abs(parent) = 3
                THEN (parent & 255)::REAL / 100
                ELSE (
                    SELECT COALESCE(SUM(t.dx * r.equity) / NULLIF(SUM(t.dx), 0), 0)
                    FROM {} t
                    JOIN {} r ON t.next = r.abs
                 WHERE t.prev = parent) END; END; $$
            LANGUAGE plpgsql;",
                abstraction(),
                abstraction(),
                isomorphism(),
                transitions(),
                abstraction()
            ))
        })
    }

    fn indices() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            leaked(format!(
                "CREATE INDEX IF NOT EXISTS idx_{0}_abs ON {0} (abs);
             CREATE INDEX IF NOT EXISTS idx_{0}_st  ON {0} (street);
             CREATE INDEX IF NOT EXISTS idx_{0}_eq  ON {0} (equity);
             CREATE INDEX IF NOT EXISTS idx_{0}_pop ON {0} (population);",
                abstraction()
            ))
        })
    }

    fn copy() -> &'static str {
        unimplemented!("Abstraction is derived, not loaded from files")
    }

    fn truncates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| leaked(format!("TRUNCATE TABLE {};", abstraction())))
    }

    fn freeze() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            leaked(format!(
                "ALTER TABLE {} SET (fillfactor = 100);
            ALTER TABLE {} SET (autovacuum_enabled = false);",
                abstraction(),
                abstraction()
            ))
        })
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        unimplemented!("Abstraction is derived, not loaded from files")
    }
}

impl Derive for Abstraction {
    fn exhaust() -> Vec<Self> {
        Street::all()
            .iter()
            .rev()
            .copied()
            .flat_map(Abstraction::all)
            .collect()
    }

    fn inserts(&self) -> String {
        let abs = i16::from(*self);
        format!(
            "INSERT INTO {} (abs, street, equity, population) VALUES ({}, get_street_abs({}::SMALLINT), get_equity({}::SMALLINT), get_population({}::SMALLINT));",
            abstraction(),
            abs,
            abs,
            abs,
            abs
        )
    }
}
