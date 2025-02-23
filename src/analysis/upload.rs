use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::Save;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::Error as E;

// some arbitrary counts for full deck:
// blueprint    ~ 154M, (grows with number of CFR iterations)
// isomorphism  ~ 139M,
// metric       ~ 40K,
// transition   ~ 29K,
// abstraction  ~ 500,

// the COPY FROM will only work if the Postgres process is running on the same machine
// and with access to the same filesystem. we should use tokio_postgrs::copy_in() to stream data.

pub struct Upload(Arc<Client>);

impl From<Arc<Client>> for Upload {
    fn from(client: Arc<Client>) -> Self {
        Self(client)
    }
}

impl Upload {
    pub async fn new() -> Self {
        Self(crate::db().await)
    }

    pub async fn upload() -> Result<(), E> {
        let this = Self::from(crate::db().await);
        this.recreate().await?;
        this.copy_metric().await?;
        this.copy_isomorphism().await?;
        this.copy_transitions().await?;
        this.make_abstraction().await?;
        this.copy_blueprint().await?;
        this.make_streets().await?;
        this.vacuum().await?;
        Ok(())
    }

    async fn recreate(&self) -> Result<(), E> {
        self.0.batch_execute(CREATE).await
    }

    async fn vacuum(&self) -> Result<(), E> {
        self.0.batch_execute(VACUUM).await
    }

    async fn has_rows(&self, table: &str) -> Result<bool, E> {
        if self.does_exist(table).await? {
            Ok(0 != self
                .0
                .query_one(&HAS_ROWS.replace("$1", table), &[])
                .await?
                .get::<_, i64>(0))
        } else {
            Ok(false)
        }
    }

    async fn does_exist(&self, table: &str) -> Result<bool, E> {
        Ok(1 == self
            .0
            .query_one(DOES_EXIST, &[&table])
            .await?
            .get::<_, i64>(0))
    }

    async fn copy_metric(&self) -> Result<(), E> {
        if self.has_rows("metric").await? {
            log::info!("tables data already uploaded (metric)");
            Ok(())
        } else {
            log::info!("copying metric data");
            self.0.batch_execute(copy_metric_sql().as_str()).await
        }
    }

    async fn copy_isomorphism(&self) -> Result<(), E> {
        if self.has_rows("isomorphism").await? {
            log::info!("tables data already uploaded (isomorphism)");
            Ok(())
        } else {
            log::info!("copying isomorphism data");
            self.0
                .batch_execute(copy_isomorphism_sql().as_str())
                .await?;
            self.0.batch_execute(&SORT_ISOMORPHISM).await
        }
    }

    async fn copy_blueprint(&self) -> Result<(), E> {
        if self.has_rows("blueprint").await? {
            log::info!("tables data already uploaded (blueprint)");
            Ok(())
        } else {
            log::info!("copying blueprint data");
            self.0.batch_execute(copy_blueprint_sql().as_str()).await
        }
    }

    async fn copy_transitions(&self) -> Result<(), E> {
        if self.has_rows("transitions").await? {
            log::info!("tables data already uploaded (transition)");
            Ok(())
        } else {
            log::info!("copying transition data");
            self.0.batch_execute(copy_transitions_sql().as_str()).await
        }
    }

    async fn make_abstraction(&self) -> Result<(), E> {
        if self.has_rows("abstraction").await? {
            log::info!("tables data already uploaded (abstraction)");
            Ok(())
        } else {
            log::info!("deriving abstraction data");
            self.0.batch_execute(MAKE_ABSTRACTION).await?;
            self.0.batch_execute(DEF_EQUITY).await?;
            self.0.batch_execute(DEF_STREET_ABS).await?;
            self.0.batch_execute(DEF_POPULATION).await?;
            self.0.batch_execute(DEF_CENTRALITY).await?;
            for (abs, street) in Street::all()
                .into_iter()
                .rev()
                .map(|&s| Abstraction::all(s).into_iter().map(move |a| (a, s)))
                .flatten()
                .map(|(abs, s)| (i64::from(abs), s as i16))
            {
                self.0.execute(SET_ABSTRACTED, &[&abs, &street]).await?;
            }
            Ok(())
        }
    }

    async fn make_streets(&self) -> Result<(), E> {
        if self.has_rows("street").await? {
            log::info!("tables data already uploaded (street)");
            Ok(())
        } else {
            log::info!("copying street data");
            self.0.batch_execute(MAKE_STREETS).await
        }
    }
}

const VACUUM: &'static str = "
VACUUM ANALYZE;
";

const HAS_ROWS: &'static str = "
SELECT  1
FROM    $1
LIMIT   1;
";

const DOES_EXIST: &'static str = "
SELECT  1
FROM    information_schema.tables
WHERE   table_name = $1;
";

const CREATE: &'static str = "
CREATE TABLE IF NOT EXISTS isomorphism (
    obs        BIGINT,
    abs        BIGINT,
    position   INTEGER
);
CREATE TABLE IF NOT EXISTS abstraction (
    abs        BIGINT,
    street     SMALLINT,
    population INTEGER,
    centrality REAL,
    equity     REAL
);
CREATE TABLE IF NOT EXISTS transitions (
    prev       BIGINT,
    next       BIGINT,
    dx         REAL
);
CREATE TABLE IF NOT EXISTS metric      (
    xor        BIGINT,
    dx         REAL
);
CREATE TABLE IF NOT EXISTS street      (
    street     SMALLINT,
    nobs       INTEGER,
    nabs       INTEGER
);
CREATE TABLE IF NOT EXISTS blueprint   (
    edge       BIGINT,
    past       BIGINT,
    present    BIGINT,
    future     BIGINT,
    policy     REAL,
    regret     REAL
);";

fn copy_metric_sql() -> String {
    format!(
        "
        TRUNCATE TABLE  metric;
        ALTER TABLE     metric SET UNLOGGED;
        COPY        metric (xor, dx) FROM '$1/{}'   WITH (FORMAT BINARY);
        COPY        metric (xor, dx) FROM '$1/{}'   WITH (FORMAT BINARY);
        COPY        metric (xor, dx) FROM '$1/{}'   WITH (FORMAT BINARY);
        COPY        metric (xor, dx) FROM '$1/{}'   WITH (FORMAT BINARY);
        INSERT INTO metric (xor, dx) VALUES (0, 0);
        CREATE INDEX IF NOT EXISTS idx_metric_xor  ON metric (xor);
        CREATE INDEX IF NOT EXISTS idx_metric_dx   ON metric (dx);
",
        crate::clustering::metric::Metric::path(Street::Rive),
        crate::clustering::metric::Metric::path(Street::Turn),
        crate::clustering::metric::Metric::path(Street::Flop),
        crate::clustering::metric::Metric::path(Street::Pref)
    )
    .replace("$1", path().as_str())
}

fn copy_isomorphism_sql() -> String {
    format!(
        "
        TRUNCATE TABLE isomorphism;
        ALTER TABLE    isomorphism SET UNLOGGED;
        COPY isomorphism (obs, abs) FROM '$1/{}' WITH (FORMAT BINARY);
        COPY isomorphism (obs, abs) FROM '$1/{}' WITH (FORMAT BINARY);
        COPY isomorphism (obs, abs) FROM '$1/{}' WITH (FORMAT BINARY);
        COPY isomorphism (obs, abs) FROM '$1/{}' WITH (FORMAT BINARY);
        CREATE INDEX IF NOT EXISTS idx_isomorphism_covering     ON isomorphism  (obs, abs) INCLUDE (abs);
        CREATE INDEX IF NOT EXISTS idx_isomorphism_abs_position ON isomorphism  (abs, position);
        CREATE INDEX IF NOT EXISTS idx_isomorphism_abs_obs      ON isomorphism  (abs, obs);
        CREATE INDEX IF NOT EXISTS idx_isomorphism_abs          ON isomorphism  (abs);
        CREATE INDEX IF NOT EXISTS idx_isomorphism_obs          ON isomorphism  (obs);
",
        crate::mccfr::encoder::Encoder::path(Street::Rive),
        crate::mccfr::encoder::Encoder::path(Street::Turn),
        crate::mccfr::encoder::Encoder::path(Street::Flop),
        crate::mccfr::encoder::Encoder::path(Street::Pref)
    )
    .replace("$1", path().as_str())
}

fn copy_blueprint_sql() -> String {
    format!(
        "
        TRUNCATE TABLE blueprint;
        ALTER TABLE    blueprint SET UNLOGGED;
        COPY           blueprint (past, present, future, edge, policy, regret)
        FROM           '$1/{}' WITH (FORMAT BINARY);
        CREATE INDEX IF NOT EXISTS idx_blueprint_bucket  ON blueprint (present, past, future);
        CREATE INDEX IF NOT EXISTS idx_blueprint_future  ON blueprint (future);
        CREATE INDEX IF NOT EXISTS idx_blueprint_present ON blueprint (present);
        CREATE INDEX IF NOT EXISTS idx_blueprint_edge    ON blueprint (edge);
        CREATE INDEX IF NOT EXISTS idx_blueprint_past    ON blueprint (past);
",
        crate::mccfr::profile::Profile::name()
    )
    .replace("$1", path().as_str())
}

fn copy_transitions_sql() -> String {
    format!(
        "
        TRUNCATE TABLE transitions;
        ALTER TABLE    transitions SET UNLOGGED;
        COPY transitions (prev, next, dx) FROM '$1/{}'   WITH (FORMAT BINARY);
        COPY transitions (prev, next, dx) FROM '$1/{}'   WITH (FORMAT BINARY);
        COPY transitions (prev, next, dx) FROM '$1/{}'   WITH (FORMAT BINARY);
        COPY transitions (prev, next, dx) FROM '$1/{}'   WITH (FORMAT BINARY);
        CREATE INDEX IF NOT EXISTS idx_transitions_dx           ON transitions(dx);
        CREATE INDEX IF NOT EXISTS idx_transitions_prev_dx      ON transitions(prev, dx);
        CREATE INDEX IF NOT EXISTS idx_transitions_next_dx      ON transitions(next, dx);
        CREATE INDEX IF NOT EXISTS idx_transitions_prev_next    ON transitions(prev, next);
        CREATE INDEX IF NOT EXISTS idx_transitions_next_prev    ON transitions(next, prev);
",
        crate::clustering::transitions::Decomp::path(Street::Rive),
        crate::clustering::transitions::Decomp::path(Street::Turn),
        crate::clustering::transitions::Decomp::path(Street::Flop),
        crate::clustering::transitions::Decomp::path(Street::Pref)
    )
    .replace("$1", path().as_str())
}

const SORT_ISOMORPHISM: &'static str = "
WITH numbered AS (
    SELECT obs,
           abs,
           row_number() OVER (PARTITION BY abs ORDER BY obs) - 1 as rn
    FROM isomorphism
)
    UPDATE isomorphism
    SET    position = numbered.rn
    FROM   numbered
    WHERE  isomorphism.obs = numbered.obs
    AND    isomorphism.abs = numbered.abs;
";

const MAKE_ABSTRACTION: &'static str = "
TRUNCATE TABLE  abstraction;
ALTER TABLE     abstraction SET UNLOGGED;
CREATE INDEX IF NOT EXISTS idx_abstraction_abs ON abstraction (abs);
CREATE INDEX IF NOT EXISTS idx_abstraction_st  ON abstraction (street);
CREATE INDEX IF NOT EXISTS idx_abstraction_eq  ON abstraction (equity);
CREATE INDEX IF NOT EXISTS idx_abstraction_pop ON abstraction (population);
CREATE INDEX IF NOT EXISTS idx_abstraction_cen ON abstraction (centrality);
";

const MAKE_STREETS: &'static str = r#"
TRUNCATE TABLE  street;
ALTER TABLE     street SET UNLOGGED;
INSERT INTO     street (street, nobs, nabs) VALUES
(0,
    (SELECT COUNT(*) FROM isomorphism e
    JOIN abstraction a ON e.abs = a.abs
    WHERE a.street = 0),
    (SELECT COUNT(*) FROM abstraction a
    WHERE a.street = 0)),
(1,
    (SELECT COUNT(*) FROM isomorphism e
    JOIN abstraction a ON e.abs = a.abs
    WHERE a.street = 1),
    (SELECT COUNT(*) FROM abstraction a
    WHERE a.street = 1)),
(2,
    (SELECT COUNT(*) FROM isomorphism e
    JOIN abstraction a ON e.abs = a.abs
    WHERE a.street = 2),
    (SELECT COUNT(*) FROM abstraction a
    WHERE a.street = 2)),
(3,
    (SELECT COUNT(*) FROM isomorphism e
    JOIN abstraction a ON e.abs = a.abs
    WHERE a.street = 3),
    (SELECT COUNT(*) FROM abstraction a
    WHERE a.street = 3));
CREATE INDEX IF NOT EXISTS idx_street_st ON street (street);
"#;

const SET_ABSTRACTED: &'static str = r#"
INSERT INTO abstraction (
    abs,
    street,
    equity,
    population,
    centrality
) VALUES (
    ($1),
    ($2),
    get_equity      ($1),
    get_population  ($1),
    get_centrality  ($1)
);
"#;

#[allow(unused)]
const DEF_STREET_OBS: &'static str = r#"
CREATE OR REPLACE FUNCTION get_street_obs(obs BIGINT) RETURNS SMALLINT AS
$$
DECLARE
    ncards INTEGER;
BEGIN
    SELECT COUNT(*)
    INTO ncards
    FROM (
        SELECT UNNEST(ARRAY[
            (obs >> 0)  & 255,
            (obs >> 8)  & 255,
            (obs >> 16) & 255,
            (obs >> 24) & 255,
            (obs >> 32) & 255,
            (obs >> 40) & 255,
            (obs >> 48) & 255
        ]) AS byte
    ) AS bytes;
    RETURN CASE
        WHEN ncards = 2 THEN 0  -- preflop
        WHEN ncards = 5 THEN 1  -- flop
        WHEN ncards = 6 THEN 2  -- turn
        WHEN ncards = 7 THEN 3  -- river
        ELSE NULL
    END;
END;
$$
LANGUAGE plpgsql;
"#;

const DEF_STREET_ABS: &'static str = r#"
CREATE OR REPLACE FUNCTION get_street_abs(abs BIGINT) RETURNS SMALLINT AS
$$ BEGIN RETURN (abs >> 56)::SMALLINT; END; $$
LANGUAGE plpgsql;
"#;

const DEF_EQUITY: &'static str = r#"
CREATE OR REPLACE FUNCTION get_equity(parent BIGINT) RETURNS REAL AS
$$ BEGIN RETURN CASE WHEN get_street_abs(parent) = 3
    THEN
        (parent & 255)::REAL / 100
    ELSE (
        SELECT COALESCE(SUM(t.dx * r.equity) / NULLIF(SUM(t.dx), 0), 0)
        FROM transitions t
        JOIN abstraction r ON t.next = r.abs
        WHERE                 t.prev = parent
    )
    END; END; $$
LANGUAGE plpgsql;
"#;

const DEF_POPULATION: &'static str = r#"
CREATE OR REPLACE FUNCTION
get_population(xxx BIGINT) RETURNS INTEGER AS
$$
BEGIN RETURN (
    SELECT COUNT(*)
    FROM isomorphism e 
    WHERE e.abs = xxx
); END;
$$
LANGUAGE plpgsql;
"#;

const DEF_CENTRALITY: &'static str = r#"
CREATE OR REPLACE FUNCTION get_centrality(xxx BIGINT) RETURNS REAL AS
$$
DECLARE
    numer REAL;
    denom INTEGER;
BEGIN
    SELECT
        SUM(get_population(a1.abs) * m.dx),
        SUM(get_population(a1.abs))
    INTO
        numer,
        denom
    FROM abstraction a1
    JOIN abstraction a2  ON a1.street = a2.street
    JOIN metric m        ON (a1.abs # a2.abs) = m.xor
    WHERE a1.abs = xxx   AND a1.abs != a2.abs;
    RETURN CASE
        WHEN denom IS NULL OR denom = 0
        THEN 0
        ELSE numer / denom
    END;
END;
$$
LANGUAGE plpgsql;
"#;

fn path() -> String {
    std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}
