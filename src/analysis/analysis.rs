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
use crate::Pipe;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::Error as E;

pub struct Analysis(Arc<Client>);

impl Analysis {
    pub fn new(client: Client) -> Self {
        Self(Arc::new(client))
    }

    pub async fn abasis(&self, street: Street) -> Result<Vec<Abstraction>, E> {
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
            .map(|row| row.get::<_, i64>(0).into())
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
            .pipe(Metric::from))
    }
    pub async fn abstractable(&self, obs: Observation) -> Result<Abstraction, E> {
        let iso = obs
            .pipe(Isomorphism::from)
            .pipe(Observation::from)
            .pipe(i64::from);
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
    pub async fn distribution(&self, obs: Observation) -> Result<Histogram, E> {
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
            .pipe(Histogram::from))
    }
    pub async fn similarities(&self, obs: Observation) -> Result<Vec<Observation>, E> {
        // 8d8s~6dJs7c
        let iso = obs
            .pipe(Isomorphism::from)
            .pipe(Observation::from)
            .pipe(i64::from);
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
    pub async fn abs_distance(&self, x: Observation, y: Observation) -> Result<Energy, E> {
        // dab Qh6s~QdTc6c QhQs~QdQcAc
        if x.street() != y.street() {
            return Err(E::__private_api_timeout());
        }
        if x == y {
            return Ok(0 as Energy);
        }
        let x = x
            .pipe(Isomorphism::from)
            .pipe(Observation::from)
            .pipe(i64::from);
        let y = y
            .pipe(Isomorphism::from)
            .pipe(Observation::from)
            .pipe(i64::from);
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
            self.distribution(x),
            self.distribution(y),
            self.metric(x.street().next())
        )?;
        Ok(Sinkhorn::from((hx, hy, metric)).minimize().cost())
    }

    /// call this exactly once after we've written everything to disk, namely:
    /// - blueprint
    /// - (for each street) metric
    /// - (for each street) encoder
    /// should probably add a method to assert that we're not erasing any data
    pub async fn upload(&self) -> Result<(), E> {
        self.nuke().await?;
        self.truncate().await?;
        self.recreate().await?;
        self.unlogged().await?;
        self.copy_metric().await?;
        self.copy_encoder().await?;
        self.copy_blueprint().await?;
        self.copy_abstraction().await?;
        Ok(())
    }
    async fn nuke(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                  
            DROP SCHEMA public CASCADE;
            CREATE SCHEMA public;
        "#).await?)
    }
    async fn recreate(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                  
            CREATE TABLE IF NOT EXISTS encoder     (obs  BIGINT, abs  BIGINT);
            CREATE TABLE IF NOT EXISTS metric      (xor  BIGINT, dx   REAL);
            CREATE TABLE IF NOT EXISTS abstraction (abs  BIGINT, st   SMALLINT);
            CREATE TABLE IF NOT EXISTS blueprint   (edge BIGINT, past BIGINT, present BIGINT, future BIGINT, policy REAL, regret REAL);
        "#).await?)
    }
    async fn truncate(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                  
            TRUNCATE TABLE encoder;
            TRUNCATE TABLE metric;
            TRUNCATE TABLE abstraction;
            TRUNCATE TABLE blueprint;
        "#).await?)
    }
    async fn unlogged(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                  
            ALTER TABLE encoder      SET UNLOGGED;
            ALTER TABLE metric       SET UNLOGGED;
            ALTER TABLE abstraction  SET UNLOGGED;
            ALTER TABLE blueprint    SET UNLOGGED;
        "#).await?)
    }
    async fn copy_blueprint(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                  
            COPY blueprint (past, present, future, edge, policy, regret) FROM '/Users/krukah/Code/robopoker/blueprint.profile.pgcopy' WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_blueprint_bucket  ON blueprint (present, past, future);
            CREATE INDEX IF NOT EXISTS idx_blueprint_future  ON blueprint (future);
            CREATE INDEX IF NOT EXISTS idx_blueprint_present ON blueprint (present);
            CREATE INDEX IF NOT EXISTS idx_blueprint_edge    ON blueprint (edge);
            CREATE INDEX IF NOT EXISTS idx_blueprint_past    ON blueprint (past);
        "#).await?)
    }
    async fn copy_metric(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                  
            COPY metric (xor, dx) FROM '/Users/krukah/Code/robopoker/turn.metric.pgcopy'       WITH (FORMAT BINARY);
            COPY metric (xor, dx) FROM '/Users/krukah/Code/robopoker/flop.metric.pgcopy'       WITH (FORMAT BINARY);
            COPY metric (xor, dx) FROM '/Users/krukah/Code/robopoker/preflop.metric.pgcopy'    WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_metric_xor  ON metric (xor);
            CREATE INDEX IF NOT EXISTS idx_metric_dx   ON metric (dx);
        "#).await?)
    }
    async fn copy_encoder(&self) -> Result<(), E> {
        Ok(self.0.batch_execute(r#"                                                                                  
            COPY encoder (obs, abs) FROM '/Users/krukah/Code/robopoker/river.encoder.pgcopy'   WITH (FORMAT BINARY);
            COPY encoder (obs, abs) FROM '/Users/krukah/Code/robopoker/turn.encoder.pgcopy'    WITH (FORMAT BINARY);
            COPY encoder (obs, abs) FROM '/Users/krukah/Code/robopoker/flop.encoder.pgcopy'    WITH (FORMAT BINARY);
            COPY encoder (obs, abs) FROM '/Users/krukah/Code/robopoker/preflop.encoder.pgcopy' WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_encoder_obs ON encoder (obs);
            CREATE INDEX IF NOT EXISTS idx_encoder_abs ON encoder (abs);
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
}
