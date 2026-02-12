//! Database schema implementations for domain types.
//!
//! Implements Schema/Derive traits directly on types from other crates.
//! This is possible because Schema/Derive are local to this crate.
use super::*;
use rbp_cards::*;
use rbp_gameplay::*;

impl Schema for Street {
    fn name() -> &'static str {
        STREET
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            STREET,
            " (
                street     SMALLINT,
                nobs       INTEGER,
                nabs       INTEGER
            );
            TRUNCATE TABLE ",
            STREET,
            ";
            CREATE OR REPLACE FUNCTION get_nabs(s SMALLINT) RETURNS INTEGER AS
            $$ BEGIN RETURN (SELECT COUNT(*) FROM ",
            ABSTRACTION,
            " a WHERE a.street = s); END; $$
            LANGUAGE plpgsql;"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "CREATE INDEX IF NOT EXISTS idx_",
            STREET,
            "_st ON ",
            STREET,
            " (street);"
        )
    }
    fn copy() -> &'static str {
        unimplemented!("Street is derived, not loaded from files")
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", STREET, ";")
    }
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            STREET,
            " SET (fillfactor = 100);
            ALTER TABLE ",
            STREET,
            " SET (autovacuum_enabled = false);"
        )
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
            STREET, s, n, s
        )
    }
}

impl Schema for Abstraction {
    fn name() -> &'static str {
        ABSTRACTION
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            ABSTRACTION,
            " (
                abs         SMALLINT,
                street      SMALLINT,
                population  INTEGER,
                equity      REAL
            );
            TRUNCATE TABLE ",
            ABSTRACTION,
            ";
            CREATE OR REPLACE FUNCTION get_population(xxx SMALLINT) RETURNS INTEGER AS
            $$ BEGIN RETURN (SELECT COUNT(*) FROM ",
            ISOMORPHISM,
            " e WHERE e.abs = xxx); END; $$
            LANGUAGE plpgsql;
            CREATE OR REPLACE FUNCTION get_street_abs(abs SMALLINT) RETURNS SMALLINT AS
            $$ BEGIN RETURN ((abs >> 8) & 255)::SMALLINT; END; $$
            LANGUAGE plpgsql;
            CREATE OR REPLACE FUNCTION get_equity(parent SMALLINT) RETURNS REAL AS
            $$ BEGIN RETURN CASE WHEN get_street_abs(parent) = 3
                THEN (parent & 255)::REAL / 100
                ELSE (
                    SELECT COALESCE(SUM(t.dx * r.equity) / NULLIF(SUM(t.dx), 0), 0)
                    FROM ",
            TRANSITIONS,
            " t
                    JOIN ",
            ABSTRACTION,
            " r ON t.next = r.abs
             WHERE t.prev = parent) END; END; $$
            LANGUAGE plpgsql;"
        )
    }
    fn indices() -> &'static str {
        const_format::concatcp!(
            "CREATE INDEX IF NOT EXISTS idx_",
            ABSTRACTION,
            "_abs ON ",
            ABSTRACTION,
            " (abs);
             CREATE INDEX IF NOT EXISTS idx_",
            ABSTRACTION,
            "_st  ON ",
            ABSTRACTION,
            " (street);
             CREATE INDEX IF NOT EXISTS idx_",
            ABSTRACTION,
            "_eq  ON ",
            ABSTRACTION,
            " (equity);
             CREATE INDEX IF NOT EXISTS idx_",
            ABSTRACTION,
            "_pop ON ",
            ABSTRACTION,
            " (population);"
        )
    }
    fn copy() -> &'static str {
        unimplemented!("Abstraction is derived, not loaded from files")
    }
    fn truncates() -> &'static str {
        const_format::concatcp!("TRUNCATE TABLE ", ABSTRACTION, ";")
    }
    fn freeze() -> &'static str {
        const_format::concatcp!(
            "ALTER TABLE ",
            ABSTRACTION,
            " SET (fillfactor = 100);
            ALTER TABLE ",
            ABSTRACTION,
            " SET (autovacuum_enabled = false);"
        )
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
            ABSTRACTION, abs, abs, abs, abs
        )
    }
}
