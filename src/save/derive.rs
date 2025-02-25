use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;

/// trait for deriving database tables from other tables
pub trait Derive: Sized {
    fn name() -> String;
    fn exhaust() -> Vec<Self>;
    fn creates() -> String;
    fn indexes() -> String;
    fn inserts(&self) -> String;
    fn derived() -> String {
        Self::exhaust()
            .into_iter()
            .map(|r| r.inserts())
            .collect::<Vec<_>>()
            .join("\n;")
    }
}

impl Derive for Abstraction {
    fn exhaust() -> Vec<Self> {
        Street::exhaust()
            .into_iter()
            .map(|s| Self::all(s).into_iter())
            .flatten()
            .collect()
    }

    fn name() -> String {
        "abstraction".to_string()
    }

    fn creates() -> String {
        "CREATE TABLE IF NOT EXISTS abstraction (
            abs         BIGINT,
            street      SMALLINT,
            population  INTEGER,
            equity      REAL
        );
        TRUNCATE TABLE abstraction;
        ALTER TABLE abstraction SET UNLOGGED;

        CREATE OR REPLACE FUNCTION get_population(xxx BIGINT) RETURNS INTEGER AS
        $$ BEGIN RETURN (SELECT COUNT(*) FROM isomorphism e WHERE e.abs = xxx); END; $$
        LANGUAGE plpgsql;
        
        CREATE OR REPLACE FUNCTION get_street_abs(abs BIGINT) RETURNS SMALLINT AS
        $$ BEGIN RETURN (abs >> 56)::SMALLINT; END; $$
        LANGUAGE plpgsql;

        CREATE OR REPLACE FUNCTION get_equity(parent BIGINT) RETURNS REAL AS
        $$ BEGIN RETURN CASE WHEN get_street_abs(parent) = 3
            THEN (parent & 255)::REAL / 100
            ELSE (
                SELECT COALESCE(SUM(t.dx * r.equity) / NULLIF(SUM(t.dx), 0), 0)
                FROM transitions t
                JOIN abstraction r ON t.next = r.abs
                WHERE                 t.prev = parent) END; END; $$
        LANGUAGE plpgsql;
        "
        .into()
    }

    fn indexes() -> String {
        "
        CREATE INDEX IF NOT EXISTS idx_abstraction_abs ON abstraction (abs);
        CREATE INDEX IF NOT EXISTS idx_abstraction_st  ON abstraction (street);
        CREATE INDEX IF NOT EXISTS idx_abstraction_eq  ON abstraction (equity);
        CREATE INDEX IF NOT EXISTS idx_abstraction_pop ON abstraction (population);
        "
        .into()
    }

    fn inserts(&self) -> String {
        let abs = i64::from(self.clone());
        format!(
            "INSERT INTO abstraction (
                abs,
                street,
                equity,
                population
            ) VALUES (          ({}),
                get_street_abs  ({}),
                get_equity      ({}),
                get_population  ({}));",
            abs, abs, abs, abs,
        )
    }
}

impl Derive for Street {
    fn exhaust() -> Vec<Self> {
        Self::all().iter().rev().copied().collect()
    }

    fn name() -> String {
        "street".to_string()
    }

    fn creates() -> String {
        "CREATE TABLE IF NOT EXISTS street (
            street     SMALLINT,
            nobs       INTEGER,
            nabs       INTEGER
        );

        TRUNCATE TABLE street;
        ALTER TABLE street SET UNLOGGED;

        CREATE OR REPLACE FUNCTION get_niso(s SMALLINT) RETURNS INTEGER AS
        $$ BEGIN RETURN (SELECT COUNT(*) FROM isomorphism e WHERE e.street = s); END; $$
        LANGUAGE plpgsql;

        CREATE OR REPLACE FUNCTION get_nabs(s SMALLINT) RETURNS INTEGER AS
        $$ BEGIN RETURN (SELECT COUNT(*) FROM abstraction a WHERE a.street = s); END; $$
        LANGUAGE plpgsql;"
            .into()
    }

    fn indexes() -> String {
        "CREATE INDEX IF NOT EXISTS idx_street_st ON street (street);".into()
    }

    fn inserts(&self) -> String {
        let street = self.clone() as i16;
        format!(
            "INSERT INTO street (
                street,
                nobs,
                nabs
            ) VALUES (  ({}),
                get_niso({}),
                get_nabs({})
            );",
            street, street, street
        )
    }
}
