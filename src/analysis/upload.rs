use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::Error as E;

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
        let count = "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1";
        for table in [
            "street",
            "metric",
            "encoder",
            "abstraction",
            "transitions",
            // blueprint,
        ] {
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
        abs        BIGINT
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
    CREATE INDEX IF NOT EXISTS idx_encoder_obs ON encoder (obs);
    CREATE INDEX IF NOT EXISTS idx_encoder_abs ON encoder (abs);
    "#,
                    path, path, path, path
                )
                .as_str(),
            )
            .await?)
    }

    async fn copy_streets(&self) -> Result<(), E> {
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

    async fn copy_transitions(&self) -> Result<(), E> {
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
    CREATE INDEX IF NOT EXISTS idx_transitions_prev ON transitions (prev);
    CREATE INDEX IF NOT EXISTS idx_transitions_next ON transitions (next);
    CREATE INDEX IF NOT EXISTS idx_transitions_dx   ON transitions (dx);
    "#,
                    path, path, path, path
                )
                .as_str(),
            )
            .await?)
    }

    async fn copy_blueprint(&self) -> Result<(), E> {
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

    async fn copy_abstraction(&self) -> Result<(), E> {
        log::info!("copying abstraction data");
        self.get_equity().await?;
        self.get_street_abs().await?;
        self.get_population().await?;
        self.get_centrality().await?;
        self.set_abstracted().await?;
        self.0
            .batch_execute(
                r#"
    SELECT set_abstracted(3::SMALLINT);
    SELECT set_abstracted(2::SMALLINT);
    SELECT set_abstracted(1::SMALLINT);
    SELECT set_abstracted(0::SMALLINT);
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
    async fn set_abstracted(&self) -> Result<(), E> {
        log::info!("deriving abstraction fields");
        self.0
            .batch_execute(
                r#"
    CREATE OR REPLACE FUNCTION
            set_abstracted(xxx SMALLINT) RETURNS VOID AS
    $$
    BEGIN
        INSERT INTO abstraction (abs, street, equity, population, centrality)
        SELECT DISTINCT ON (e.abs)
            e.abs,
            get_street_abs(e.abs),
            get_equity(e.abs),
            get_population(e.abs),
            get_centrality(e.abs)
        FROM encoder e
        WHERE get_street_abs(e.abs) = xxx;
    END;
    $$
    LANGUAGE plpgsql;
    "#,
            )
            .await?;
        Ok(())
    }

    #[allow(unused)]
    async fn get_street_obs(&self) -> Result<(), E> {
        self.0
            .batch_execute(
                r#"
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
