#![allow(unused)]

use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::Error as E;

// encoder      ~ 140M,
// transition   ~ 10K,
// metric       ~ 40K,
// abstraction  ~ 500,

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
        if this.done().await? {
            log::info!("data already uploaded");
            Ok(())
        } else {
            log::info!("uploading data");
            this.nuke().await?;
            this.recreate().await?;
            this.truncate().await?;
            this.unlogged().await?;
            this.copy_metric().await?;
            this.copy_encoder().await?;
            this.copy_blueprint().await?;
            this.copy_transitions().await?;
            this.copy_abstraction().await?;
            this.copy_streets().await?;
            Ok(())
        }
    }

    async fn done(&self) -> Result<bool, E> {
        return Ok(true);
        log::info!("checking if data is uploaded");
        for table in [
            "street",
            "metric",
            "encoder",
            "abstraction",
            "transitions",
            // blueprint,
        ] {
            let count = "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1";
            if 0 == self.0.query_one(count, &[&table]).await?.get::<_, i64>(0) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn nuke(&self) -> Result<(), E> {
        log::info!("nuking database schema (not really)");
        return Ok(());
        #[allow(unreachable_code)]
        Ok(self
            .0
            .batch_execute(
                r#"
                    DROP SCHEMA public CASCADE;
                    CREATE SCHEMA public;
    "#,
            )
            .await?)
    }

    async fn recreate(&self) -> Result<(), E> {
        log::info!("creating tables");
        Ok(self
            .0
            .batch_execute(
                r#"
                    CREATE TABLE IF NOT EXISTS street      (
                        street     SMALLINT,
                        nobs       INTEGER,
                        nabs       INTEGER
                    );
                    CREATE TABLE IF NOT EXISTS encoder     (
                        obs        BIGINT,
                        abs        BIGINT,
                        position   INTEGER
                    );
                    CREATE TABLE IF NOT EXISTS metric      (
                        xor        BIGINT,
                        dx         REAL
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
                    CREATE TABLE IF NOT EXISTS blueprint   (
                        edge       BIGINT,
                        past       BIGINT,
                        present    BIGINT,
                        future     BIGINT,
                        policy     REAL,
                        regret     REAL
                    );"#,
            )
            .await?)
    }

    async fn truncate(&self) -> Result<(), E> {
        log::info!("truncating all tables");
        Ok(self
            .0
            .batch_execute(
                r#"
                    TRUNCATE TABLE encoder;
                    TRUNCATE TABLE metric;
                    TRUNCATE TABLE abstraction;
                    TRUNCATE TABLE transitions;
                    TRUNCATE TABLE street;
                    TRUNCATE TABLE blueprint;
    "#,
            )
            .await?)
    }

    async fn unlogged(&self) -> Result<(), E> {
        log::info!("setting tables to unlogged");
        Ok(self
            .0
            .batch_execute(
                r#"
                    ALTER TABLE encoder      SET UNLOGGED;
                    ALTER TABLE metric       SET UNLOGGED;
                    ALTER TABLE abstraction  SET UNLOGGED;
                    ALTER TABLE transitions  SET UNLOGGED;
                    ALTER TABLE street       SET UNLOGGED;
                    ALTER TABLE blueprint    SET UNLOGGED;
    "#,
            )
            .await?)
    }

    async fn copy_metric(&self) -> Result<(), E> {
        return Ok(());
        log::info!("copying metric data");
        let path = self.path();
        Ok(self
            .0
            .batch_execute(
                format!(
                    r#"
                        INSERT INTO metric (xor, dx) VALUES (0, 0);
                        COPY        metric (xor, dx) FROM '{}/pgcopy.metric.river'      WITH (FORMAT BINARY);
                        COPY        metric (xor, dx) FROM '{}/pgcopy.metric.turn'       WITH (FORMAT BINARY);
                        COPY        metric (xor, dx) FROM '{}/pgcopy.metric.flop'       WITH (FORMAT BINARY);
                        COPY        metric (xor, dx) FROM '{}/pgcopy.metric.preflop'    WITH (FORMAT BINARY);
                        CREATE INDEX IF NOT EXISTS idx_metric_xor  ON metric (xor);
                        CREATE INDEX IF NOT EXISTS idx_metric_dx   ON metric (dx);
    "#,
                    path, path, path, path
                )
                .as_str(),
            )
            .await?)
    }

    async fn copy_encoder(&self) -> Result<(), E> {
        return Ok(());
        log::info!("copying observation data");
        let path = self.path();
        Ok(self
            .0
            .batch_execute(
                format!(
                    r#"
                        COPY encoder (obs, abs) FROM '{}/pgcopy.encoder.river'   WITH (FORMAT BINARY);
                        COPY encoder (obs, abs) FROM '{}/pgcopy.encoder.turn'    WITH (FORMAT BINARY);
                        COPY encoder (obs, abs) FROM '{}/pgcopy.encoder.flop'    WITH (FORMAT BINARY);
                        COPY encoder (obs, abs) FROM '{}/pgcopy.encoder.preflop' WITH (FORMAT BINARY);

                        CREATE INDEX IF NOT EXISTS idx_encoder_abs_obs ON encoder (abs, obs);
                        CREATE INDEX IF NOT EXISTS idx_encoder_abs_position ON encoder(abs, position);
                        CREATE INDEX IF NOT EXISTS idx_encoder_abs ON encoder (abs); -- drop ?
                        CREATE INDEX IF NOT EXISTS idx_encoder_obs ON encoder (obs);
                        
                        WITH numbered AS (
                            SELECT obs,
                                   abs,
                                   row_number() OVER (PARTITION BY abs ORDER BY obs) - 1 as rn
                            FROM encoder
                        )
                        UPDATE encoder
                        SET    position = numbered.rn
                        FROM   numbered
                        WHERE  encoder.obs = numbered.obs
                        AND    encoder.abs = numbered.abs;
    "#,
                    path, path, path, path
                )
                .as_str(),
            )
            .await?)
    }

    async fn copy_blueprint(&self) -> Result<(), E> {
        return Ok(());
        log::info!("copying blueprint data");
        let path = self.path();
        Ok(self.0.batch_execute(format!(r#"
            COPY blueprint (past, present, future, edge, policy, regret) FROM '{}/pgcopy.profile.blueprint' WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_blueprint_bucket  ON blueprint (present, past, future);
            CREATE INDEX IF NOT EXISTS idx_blueprint_future  ON blueprint (future);
            CREATE INDEX IF NOT EXISTS idx_blueprint_present ON blueprint (present);
            CREATE INDEX IF NOT EXISTS idx_blueprint_edge    ON blueprint (edge);
            CREATE INDEX IF NOT EXISTS idx_blueprint_past    ON blueprint (past);
    "#, path).as_str()).await?)
    }

    async fn copy_transitions(&self) -> Result<(), E> {
        return Ok(());
        log::info!("copying transition data");
        let path = self.path();
        Ok(self
            .0
            .batch_execute(
                format!(
                    r#"
                        COPY transitions (prev, next, dx) FROM '{}/pgcopy.transitions.river'   WITH (FORMAT BINARY);
                        COPY transitions (prev, next, dx) FROM '{}/pgcopy.transitions.turn'    WITH (FORMAT BINARY);
                        COPY transitions (prev, next, dx) FROM '{}/pgcopy.transitions.flop'    WITH (FORMAT BINARY);
                        COPY transitions (prev, next, dx) FROM '{}/pgcopy.transitions.preflop' WITH (FORMAT BINARY);
                        CREATE INDEX IF NOT EXISTS idx_transitions_dx           ON transitions(dx);
                        CREATE INDEX IF NOT EXISTS idx_transitions_prev_dx      ON transitions(prev, dx);
                        CREATE INDEX IF NOT EXISTS idx_transitions_next_dx      ON transitions(next, dx);
                        CREATE INDEX IF NOT EXISTS idx_transitions_prev_next    ON transitions(prev, next);
                        CREATE INDEX IF NOT EXISTS idx_transitions_next_prev    ON transitions(next, prev);
    "#,
                    path, path, path, path
                )
                .as_str(),
            )
            .await?)
    }

    async fn copy_abstraction(&self) -> Result<(), E> {
        return Ok(());
        log::info!("deriving abstraction data");
        self.get_equity().await?;
        self.get_street_abs().await?;
        self.get_population().await?;
        self.get_centrality().await?;
        self.set_abstracted().await?;
        self.0
            .batch_execute(
                r#"
                    CREATE INDEX IF NOT EXISTS idx_abstraction_abs ON abstraction (abs);
                    CREATE INDEX IF NOT EXISTS idx_abstraction_st  ON abstraction (street);
                    CREATE INDEX IF NOT EXISTS idx_abstraction_eq  ON abstraction (equity);
                    CREATE INDEX IF NOT EXISTS idx_abstraction_pop ON abstraction (population);
                    CREATE INDEX IF NOT EXISTS idx_abstraction_cen ON abstraction (centrality);
    "#,
            )
            .await?;
        Ok(())
    }

    async fn copy_streets(&self) -> Result<(), E> {
        return Ok(());
        log::info!("copying street data");
        Ok(self
            .0
            .batch_execute(
                r#"
                    INSERT INTO street (street, nobs, nabs) VALUES
                    (0,
                        (SELECT COUNT(*) FROM encoder e
                        JOIN abstraction a ON e.abs = a.abs
                        WHERE a.street = 0),
                        (SELECT COUNT(*) FROM abstraction a
                        WHERE a.street = 0)),
                    (1,
                        (SELECT COUNT(*) FROM encoder e
                        JOIN abstraction a ON e.abs = a.abs
                        WHERE a.street = 1),
                        (SELECT COUNT(*) FROM abstraction a
                        WHERE a.street = 1)),
                    (2,
                        (SELECT COUNT(*) FROM encoder e
                        JOIN abstraction a ON e.abs = a.abs
                        WHERE a.street = 2),
                        (SELECT COUNT(*) FROM abstraction a
                        WHERE a.street = 2)),
                    (3,
                        (SELECT COUNT(*) FROM encoder e
                        JOIN abstraction a ON e.abs = a.abs
                        WHERE a.street = 3),
                        (SELECT COUNT(*) FROM abstraction a
                        WHERE a.street = 3));
                    CREATE INDEX IF NOT EXISTS idx_street_st ON street (street);
    "#,
            )
            .await?)
    }

    async fn set_abstracted(&self) -> Result<(), E> {
        for (abs, street) in Street::all()
            .into_iter()
            .rev()
            .map(|&s| Abstraction::all(s).into_iter().map(move |a| (a, s)))
            .flatten()
            .map(|(abs, s)| (i64::from(abs), s as i16))
        {
            self.0
                .execute(
                    r#"
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
                        )
                    "#,
                    &[&abs, &street],
                )
                .await?;
        }
        Ok(())
    }

    #[allow(unused)]
    async fn get_street_obs(&self) -> Result<(), E> {
        self.0
            .batch_execute(
                r#"

    -- get the street from an observation
    -- by counting the number of cards

    CREATE OR REPLACE FUNCTION
    get_street_obs(obs BIGINT) RETURNS SMALLINT AS
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
    "#,
            )
            .await?;
        Ok(())
    }
    async fn get_street_abs(&self) -> Result<(), E> {
        self.0
            .batch_execute(
                r#"

    -- get the street from an abstraction
    -- by extracting highest 8 MSBs

    CREATE OR REPLACE FUNCTION
    get_street_abs(abs BIGINT) RETURNS SMALLINT AS
    $$
    BEGIN RETURN (abs >> 56)::SMALLINT; END;
    $$
    LANGUAGE plpgsql;
    "#,
            )
            .await?;
        Ok(())
    }
    async fn get_equity(&self) -> Result<(), E> {
        self.0
            .batch_execute(
                r#"

    -- reucrsively calculate equity
    -- by integrating over the
    -- transition density matrix

    CREATE OR REPLACE FUNCTION
    get_equity(parent BIGINT) RETURNS REAL AS
    $$
    BEGIN
        RETURN CASE
            WHEN get_street_abs(parent) = 3
            THEN
                (parent & 255)::REAL / 100
            ELSE (
                SELECT COALESCE(SUM(t.dx * r.equity) / NULLIF(SUM(t.dx), 0), 0)
                FROM transitions t
                JOIN abstraction r ON t.next = r.abs
                WHERE t.prev = parent
            )
        END;
    END;
    $$
    LANGUAGE plpgsql;
    "#,
            )
            .await?;
        Ok(())
    }
    async fn get_population(&self) -> Result<(), E> {
        self.0
            .batch_execute(
                r#"

    -- get the population of an abstraction
    -- by counting the number of observations

    CREATE OR REPLACE FUNCTION
    get_population(xxx BIGINT) RETURNS INTEGER AS
    $$
    BEGIN RETURN ( SELECT COUNT(*) FROM encoder e WHERE e.abs = xxx ); END;
    $$
    LANGUAGE plpgsql;
    "#,
            )
            .await?;
        Ok(())
    }
    async fn get_centrality(&self) -> Result<(), E> {
        self.0
            .batch_execute(
                r#"

    -- get the absolute mean distance
    -- of a given abstraction to all others
    -- as a measure of outlierhood

    CREATE OR REPLACE FUNCTION
    get_centrality(xxx BIGINT) RETURNS REAL AS
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
    "#,
            )
            .await?;
        Ok(())
    }

    fn path(&self) -> String {
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }
}
