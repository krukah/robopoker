use crate::cards::isomorphism::Isomorphism;
use crate::cards::observation::Observation;
use crate::cards::street::Street;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::metric::Metric;
use crate::clustering::pair::Pair;
use crate::clustering::sinkhorn::Sinkhorn;
use crate::transport::coupling::Coupling;
use crate::Energy;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::Error as E;

pub struct Analysis(Arc<Client>);

impl Analysis {
    pub async fn new() -> Self {
        log::info!("connecting to db (Analysis)");
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

    pub async fn upload(&self) -> Result<(), E> {
        if self.done().await? {
            log::info!("data already uploaded");
            Ok(())
        } else {
            log::info!("uploading data");
            self.nuke().await?;
            self.recreate().await?;
            self.truncate().await?;
            self.unlogged().await?;
            self.copy_metric().await?;
            self.copy_encoder().await?;
            self.copy_streets().await?;
            self.copy_blueprint().await?;
            self.copy_abstraction().await?;
            self.copy_transitions().await?;
            Ok(())
        }
    }

    pub async fn basis(&self, street: Street) -> Result<Vec<Abstraction>, E> {
        let street = street as i16;
        const SQL: &'static str = r#"
            SELECT a2.abs
            FROM abstraction a2
            JOIN abstraction a1 ON a2.st = a1.st
            WHERE a1.abs = $1;
        "#;
        Ok(self
            .0
            .query(SQL, &[&street])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Abstraction::from)
            .collect())
    }
    pub async fn metric(&self, street: Street) -> Result<Metric, E> {
        let street = street as i16;
        const SQL: &'static str = r#"
            SELECT
                a1.abs # a2.abs AS xor,
                m.dx            AS dx
            FROM abstraction a1
            JOIN abstraction a2
                ON a1.st = a2.st
            JOIN metric m
                ON (a1.abs # a2.abs) = m.xor
            WHERE
                a1.st   = $1 AND
                a1.abs != a2.abs;
        "#;
        Ok(self
            .0
            .query(SQL, &[&street])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, Energy>(1)))
            .map(|(xor, distance)| (Pair::from(xor), distance))
            .collect::<BTreeMap<Pair, Energy>>()
            .into())
    }

    pub async fn abstraction(&self, obs: Observation) -> Result<Abstraction, E> {
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        const SQL: &'static str = r#"
            SELECT abs
            FROM encoder
            WHERE obs = $1
        "#;
        Ok(self
            .0
            .query_one(SQL, &[&iso])
            .await?
            .get::<_, i64>(0)
            .into())
    }

    pub async fn isomorphisms(&self, obs: Observation) -> Result<Vec<Observation>, E> {
        // 8d8s~6dJs7c
        let iso = i64::from(Observation::from(Isomorphism::from(obs)));
        const SQL: &'static str = r#"
            SELECT obs
            FROM encoder
            WHERE abs = (
                SELECT abs
                FROM encoder
                WHERE obs = $1
            )
            AND obs != $1
            ORDER BY RANDOM()
            LIMIT 5;
        "#;
        Ok(self
            .0
            .query(SQL, &[&iso])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }
    pub async fn constituents(&self, abs: Abstraction) -> Result<Vec<Observation>, E> {
        let abs = i64::from(abs);
        const SQL: &'static str = r#"
            SELECT obs
            FROM encoder
            WHERE abs = $1
            ORDER BY RANDOM()
            LIMIT 5;
        "#;
        Ok(self
            .0
            .query(SQL, &[&abs])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }
    pub async fn neighborhood(&self, abs: Abstraction) -> Result<Vec<(Abstraction, Energy)>, E> {
        let abs = i64::from(abs);
        const SQL: &'static str = r#"
            SELECT a1.abs, m.dx
            FROM abstraction a1
            JOIN abstraction a2 ON a1.st = a2.st
            JOIN metric m ON (a1.abs # $1) = m.xor
            WHERE
                a2.abs  = $1 AND
                a1.abs != $1
            ORDER BY m.dx
            LIMIT 5;
        "#;
        Ok(self
            .0
            .query(SQL, &[&abs])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }

    pub async fn abs_histogram(&self, abs: Abstraction) -> Result<Histogram, E> {
        let mass = abs.street().n_children() as f32;
        let abs = i64::from(abs);
        const SQL: &'static str = r#"
            SELECT next, dx
            FROM transitions
            WHERE prev = $1
        "#;
        Ok(self
            .0
            .query(SQL, &[&abs])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, Energy>(1)))
            .map(|(next, dx)| (next, (dx * mass).round() as usize))
            .map(|(next, dx)| (Abstraction::from(next), dx))
            .fold(Histogram::default(), |mut h, (next, dx)| {
                h.set(next, dx);
                h
            }))
    }
    pub async fn obs_histogram(&self, obs: Observation) -> Result<Histogram, E> {
        // Kd8s~6dJsAc
        let isos = obs
            .children()
            .map(Isomorphism::from)
            .map(Observation::from)
            .map(|obs| i64::from(obs))
            .collect::<BTreeSet<i64>>()
            .into_iter()
            .collect::<Vec<i64>>();
        const SQL: &'static str = r#"
            SELECT abs
            FROM encoder
            WHERE obs = ANY($1)
        "#;
        Ok(self
            .0
            .query(SQL, &[&isos])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Abstraction::from)
            .collect::<Vec<Abstraction>>()
            .into())
    }

    pub async fn abs_distance(&self, x: Observation, y: Observation) -> Result<Energy, E> {
        // dab Qh6s~QdTc6c QhQs~QdQcAc
        if x.street() != y.street() {
            return Err(E::__private_api_timeout());
        }
        if x == y {
            return Ok(0 as Energy);
        }
        let x = i64::from(Observation::from(Isomorphism::from(x)));
        let y = i64::from(Observation::from(Isomorphism::from(y)));
        const SQL: &'static str = r#"
            SELECT m.dx
            FROM encoder e1
            JOIN encoder e2
                ON  e1.obs = $1
                AND e2.obs = $2
            JOIN metric m
                ON (e1.abs # e2.abs) = m.xor;
        "#;
        Ok(self.0.query_one(SQL, &[&x, &y]).await?.get::<_, Energy>(0))
    }
    pub async fn obs_distance(&self, x: Observation, y: Observation) -> Result<Energy, E> {
        // dob Kd8s~6dJsAc QhQs~QdQcAc
        if x.street() != y.street() {
            return Err(E::__private_api_timeout());
        }
        let (ref hx, ref hy, ref metric) = tokio::try_join!(
            self.obs_histogram(x),
            self.obs_histogram(y),
            self.metric(x.street().next())
        )?;
        Ok(Sinkhorn::from((hx, hy, metric)).minimize().cost())
    }

    fn path(&self) -> String {
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }
    async fn done(&self) -> Result<bool, E> {
        for table in vec!["street", "metric", "encoder", "abstraction", "transitions"] {
            let count: i64 = self
                .0
                .query_one(
                    "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1",
                    &[&table],
                )
                .await?
                .get(0);
            if count == 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }
    async fn nuke(&self) -> Result<(), E> {
        return Ok(());
        #[allow(unreachable_code)]
        Ok(self.0.batch_execute(r#"                                                                                                   
            DROP SCHEMA public CASCADE;
            CREATE SCHEMA public;
        "#).await?)
    }
    async fn recreate(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                        
            CREATE TABLE IF NOT EXISTS street      (st SMALLINT, nobs INTEGER, nabs INTEGER);
            CREATE TABLE IF NOT EXISTS encoder     (obs  BIGINT, abs  BIGINT);
            CREATE TABLE IF NOT EXISTS metric      (xor  BIGINT, dx   REAL);
            CREATE TABLE IF NOT EXISTS abstraction (abs  BIGINT, st   SMALLINT);
            CREATE TABLE IF NOT EXISTS transitions (prev BIGINT, next BIGINT, dx   REAL);
            CREATE TABLE IF NOT EXISTS blueprint   (edge BIGINT, past BIGINT, present BIGINT, future BIGINT, policy REAL, regret REAL);
        "#).await?)
    }
    async fn truncate(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                                   
            TRUNCATE TABLE encoder;
            TRUNCATE TABLE metric;
            TRUNCATE TABLE abstraction;
            TRUNCATE TABLE transitions;
            TRUNCATE TABLE street;
            TRUNCATE TABLE blueprint;
        "#).await?)
    }
    async fn unlogged(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                                   
            ALTER TABLE encoder      SET UNLOGGED;
            ALTER TABLE metric       SET UNLOGGED;
            ALTER TABLE abstraction  SET UNLOGGED;
            ALTER TABLE transitions  SET UNLOGGED;
            ALTER TABLE street       SET UNLOGGED;
            ALTER TABLE blueprint    SET UNLOGGED;
        "#).await?)
    }
    async fn copy_metric(&self) -> Result<(), E> {
        let path = self.path();
        Ok(self.0.batch_execute(format!(r#"                                                                                                   
            INSERT INTO metric (xor, dx) VALUES (0, 0);
            COPY        metric (xor, dx) FROM '{}/turn.metric.pgcopy'       WITH (FORMAT BINARY);
            COPY        metric (xor, dx) FROM '{}/flop.metric.pgcopy'       WITH (FORMAT BINARY);
            COPY        metric (xor, dx) FROM '{}/preflop.metric.pgcopy'    WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_metric_xor  ON metric (xor);
            CREATE INDEX IF NOT EXISTS idx_metric_dx   ON metric (dx);
        "#, path, path, path).as_str()).await?)
    }
    async fn copy_encoder(&self) -> Result<(), E> {
        let path = self.path();
        Ok(self.0.batch_execute(format!(r#"                                                                                                   
            COPY encoder (obs, abs) FROM '{}/river.encoder.pgcopy'   WITH (FORMAT BINARY);
            COPY encoder (obs, abs) FROM '{}/turn.encoder.pgcopy'    WITH (FORMAT BINARY);
            COPY encoder (obs, abs) FROM '{}/flop.encoder.pgcopy'    WITH (FORMAT BINARY);
            COPY encoder (obs, abs) FROM '{}/preflop.encoder.pgcopy' WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_encoder_obs ON encoder (obs);
            CREATE INDEX IF NOT EXISTS idx_encoder_abs ON encoder (abs);
        "#, path, path, path, path).as_str()).await?)
    }
    async fn copy_streets(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                                                
            INSERT INTO street (st, nobs, nabs) VALUES
                (0, 
                    (SELECT COUNT(*) FROM encoder e
                    JOIN abstraction a ON e.abs = a.abs
                    WHERE a.st = 0),
                    (SELECT COUNT(*) FROM abstraction a
                    WHERE a.st = 0)),
                (1, 
                    (SELECT COUNT(*) FROM encoder e
                    JOIN abstraction a ON e.abs = a.abs
                    WHERE a.st = 1),
                    (SELECT COUNT(*) FROM abstraction a
                    WHERE a.st = 1)),
                (2, 
                    (SELECT COUNT(*) FROM encoder e
                    JOIN abstraction a ON e.abs = a.abs
                    WHERE a.st = 2),
                    (SELECT COUNT(*) FROM abstraction a
                    WHERE a.st = 2)),
                (3, 
                    (SELECT COUNT(*) FROM encoder e
                    JOIN abstraction a ON e.abs = a.abs
                    WHERE a.st = 3),
                    (SELECT COUNT(*) FROM abstraction a
                    WHERE a.st = 3));
                CREATE INDEX IF NOT EXISTS idx_street_st ON street (st);
        "#).await?)
    }
    async fn copy_abstraction(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                                   
            CREATE OR REPLACE FUNCTION street(obs BIGINT) RETURNS SMALLINT AS
            $$
            DECLARE
                obits   BIT(64);
                n_cards INTEGER := 0;
                i       INTEGER;
            BEGIN
                obits := obs::BIT(64);
                FOR i IN 0..7 LOOP
                    IF substring(obits FROM (64 - (i * 8 + 7)) FOR 8) <> B'00000000' THEN
                        n_cards := n_cards + 1;
                    END IF;
                END LOOP;
                IF    n_cards = 2 THEN RETURN 0;  -- Street::Pref
                ELSIF n_cards = 5 THEN RETURN 1;  -- Street::Flop
                ELSIF n_cards = 6 THEN RETURN 2;  -- Street::Turn
                ELSIF n_cards = 7 THEN RETURN 3;  -- Street::River
                ELSE  RAISE EXCEPTION 'invalid observation: %', n_cards;
                END IF;
            END;
            $$
            LANGUAGE plpgsql;
            INSERT INTO abstraction (abs, st)
            SELECT
                e.abs                AS abs,
                street(MIN(e.obs))   AS st
            FROM encoder e
            GROUP BY e.abs;
            CREATE INDEX IF NOT EXISTS idx_abstraction_abs ON abstraction (abs);
            CREATE INDEX IF NOT EXISTS idx_abstraction_st  ON abstraction (st);
        "#).await?)
    }
    async fn copy_transitions(&self) -> Result<(), E> {
        let path = self.path();
        Ok(self.0.batch_execute(format!(r#"                                                                                                   
            COPY transitions (prev, next, dx) FROM '{}/river.transition.pgcopy'   WITH (FORMAT BINARY);
            COPY transitions (prev, next, dx) FROM '{}/turn.transition.pgcopy'    WITH (FORMAT BINARY);
            COPY transitions (prev, next, dx) FROM '{}/flop.transition.pgcopy'    WITH (FORMAT BINARY);
            COPY transitions (prev, next, dx) FROM '{}/preflop.transition.pgcopy' WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_transitions_prev ON transitions (prev);
            CREATE INDEX IF NOT EXISTS idx_transitions_next ON transitions (next);
            CREATE INDEX IF NOT EXISTS idx_transitions_dx   ON transitions (dx);
        "#, path, path, path, path).as_str()).await?)
    }
    async fn copy_blueprint(&self) -> Result<(), E> {
        let path = self.path();
        Ok(self.0.batch_execute(format!(r#"                                                                                                   
            COPY blueprint (past, present, future, edge, policy, regret) FROM '{}/blueprint.profile.pgcopy' WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_blueprint_bucket  ON blueprint (present, past, future);
            CREATE INDEX IF NOT EXISTS idx_blueprint_future  ON blueprint (future);
            CREATE INDEX IF NOT EXISTS idx_blueprint_present ON blueprint (present);
            CREATE INDEX IF NOT EXISTS idx_blueprint_edge    ON blueprint (edge);
            CREATE INDEX IF NOT EXISTS idx_blueprint_past    ON blueprint (past);
        "#, path).as_str()).await?)
    }
}

impl From<Client> for Analysis {
    fn from(client: Client) -> Self {
        Self(Arc::new(client))
    }
}
