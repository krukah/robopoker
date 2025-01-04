use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::Error as E;

pub struct Upload(Arc<Client>);

impl Upload {
    pub async fn new() -> Self {
        log::info!("connecting to db (Upload)");
        let (client, connection) = tokio_postgres::Config::default()
            .host("localhost")
            .port(5432)
            .dbname("robopoker")
            .connect(tokio_postgres::NoTls)
            .await
            .expect("db connection");
        tokio::spawn(connection);
        Self(Arc::new(client))
    }

    pub async fn upload() -> Result<(), E> {
        let db = Self::new().await;
        if db.done().await? {
            log::info!("data already uploaded");
            Ok(())
        } else {
            log::info!("uploading data");
            db.nuke().await?;
            db.recreate().await?;
            db.truncate().await?;
            db.unlogged().await?;
            db.copy_metric().await?;
            db.copy_encoder().await?;
            db.copy_streets().await?;
            db.copy_blueprint().await?;
            db.copy_abstraction().await?;
            db.copy_transitions().await?;
            Ok(())
        }
    }

    async fn done(&self) -> Result<bool, E> {
        let count = "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1";
        for table in ["street", "metric", "encoder", "abstraction", "transitions"] {
            if 0 == self.0.query_one(count, &[&table]).await?.get(0) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn nuke(&self) -> Result<(), E> {
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
        Ok(self
            .0
            .batch_execute(
                r#"
            CREATE TABLE IF NOT EXISTS street      (
                st         SMALLINT,
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
                st         SMALLINT,
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
            );
        "#,
            )
            .await?)
    }

    async fn truncate(&self) -> Result<(), E> {
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
        let path = self.path();
        Ok(self.0.batch_execute(format!(r#"
            COPY transitions (prev, next, dx) FROM '{}/pgcopy.transitions.river'   WITH (FORMAT BINARY);
            COPY transitions (prev, next, dx) FROM '{}/pgcopy.transitions.turn'    WITH (FORMAT BINARY);
            COPY transitions (prev, next, dx) FROM '{}/pgcopy.transitions.flop'    WITH (FORMAT BINARY);
            COPY transitions (prev, next, dx) FROM '{}/pgcopy.transitions.preflop' WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_transitions_prev ON transitions (prev);
            CREATE INDEX IF NOT EXISTS idx_transitions_next ON transitions (next);
            CREATE INDEX IF NOT EXISTS idx_transitions_dx   ON transitions (dx);
        "#, path, path, path, path).as_str()).await?)
    }

    async fn copy_blueprint(&self) -> Result<(), E> {
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
        self.get_street().await?;
        self.get_equity().await?;
        self.get_population().await?;
        self.get_centrality().await?;
        self.0
            .batch_execute(
                r#"
            INSERT INTO abstraction (abs, street, equity, population, centrality)
            SELECT DISTINCT
                e.abs,
                get_street(e.abs),
                get_equity(e.abs),
                get_population(e.abs),
                get_centrality(e.abs)
            FROM encoder e;
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

    async fn get_street(&self) -> Result<(), E> {
        self.0
            .batch_execute(
                r#"
            CREATE OR REPLACE FUNCTION
                get_street(abs BIGINT) RETURNS SMALLINT AS
                $$
                BEGIN RETURN (abs >> 56)::SMALLINT; END;
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
                get_population(abs BIGINT) RETURNS INTEGER AS
                $$
                BEGIN RETURN ( SELECT COUNT(*) FROM encoder e WHERE e.abs = abs ); END;
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
                get_centrality(abs BIGINT) RETURNS REAL AS
                $$
                DECLARE
                    numer REAL;
                    denom INTEGER;
                BEGIN
                    SELECT
                        SUM(get_population(a2.abs) * m.dx),
                        SUM(get_population(a2.abs))
                    INTO
                        numer,
                        denom
                    FROM abstraction a1
                    JOIN abstraction a2  ON get_street(a1.abs) = get_street(a2.abs)
                    JOIN metric m        ON (a1.abs # a2.abs)  = m.xor
                    WHERE a1.abs = abs   AND a1.abs != a2.abs;
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

    async fn get_equity(&self) -> Result<(), E> {
        self.0
            .batch_execute(
                r#"
            CREATE OR REPLACE FUNCTION
                get_equity(abs BIGINT) RETURNS REAL AS
                $$
                DECLARE
                    street  SMALLINT;
                    numer   REAL;
                    denom   REAL;
                BEGIN
                    street   := get_street(abs);
                    IF street = 3 THEN RETURN (abs & 255)::REAL / 100; END IF;
                    SELECT
                        SUM(t.dx * get_equity(t.next)),
                        SUM(t.dx)
                    INTO
                        numer,
                        denom
                    FROM transitions t
                    WHERE t.prev = abs;
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
