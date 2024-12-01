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
use tokio_postgres::Error as PgError;

pub struct Analysis(Arc<Client>);

impl Analysis {
    pub fn new(client: Client) -> Self {
        Self(Arc::new(client))
    }

    pub async fn abstraction(&self, obs: Observation) -> Result<Abstraction, PgError> {
        let iso = obs
            .pipe(Isomorphism::from)
            .pipe(Observation::from)
            .pipe(i64::from);
        const OBS_TO_ABS: &'static str = r#"
            SELECT abs 
            FROM encoder 
            WHERE obs = $1
        "#;
        Ok(self
            .0
            .query_one(OBS_TO_ABS, &[&iso])
            .await?
            .get::<_, i64>(0)
            .into())
    }

    pub async fn histogram(&self, obs: Observation) -> Result<Histogram, PgError> {
        let isos = obs
            .children()
            .map(Isomorphism::from)
            .map(Observation::from)
            .map(|obs| i64::from(obs))
            .collect::<BTreeSet<i64>>()
            .into_iter()
            .collect::<Vec<i64>>();
        const SQL_HISTOGRAM: &'static str = r#"
            SELECT abs 
            FROM encoder 
            WHERE obs = ANY($1)
        "#;
        Ok(self
            .0
            .query(SQL_HISTOGRAM, &[&isos])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Abstraction::from)
            .collect::<Vec<Abstraction>>()
            .pipe(Histogram::from))
    }

    pub async fn neighborhood(&self, abs: Abstraction) -> Result<Vec<(Abstraction, f32)>, PgError> {
        let abs = i64::from(abs);
        const SQL_NEIGHBORHOOD: &'static str = r#"
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
            .query(SQL_NEIGHBORHOOD, &[&abs])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, Energy>(1)))
            .map(|(abs, distance)| (Abstraction::from(abs), distance))
            .collect())
    }

    pub async fn membership(&self, abs: Abstraction) -> Result<Vec<Observation>, PgError> {
        let abs = i64::from(abs);
        const SQL_MEMBERSHIP: &'static str = r#"
            SELECT obs
            FROM encoder
            WHERE abs = $1
            LIMIT 10;
        "#;
        Ok(self
            .0
            .query(SQL_MEMBERSHIP, &[&abs])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0))
            .map(Observation::from)
            .collect())
    }

    pub async fn basis(&self, street: Street) -> Result<Vec<Abstraction>, PgError> {
        let street = street as i8;
        const SQL_BASIS: &'static str = r#"
            SELECT a2.abs
            FROM abstraction a2
            JOIN abstraction a1 ON a2.st = a1.st
            WHERE a1.abs = $1;
        "#;
        Ok(self
            .0
            .query(SQL_BASIS, &[&street])
            .await?
            .iter()
            .map(|row| row.get::<_, i64>(0).into())
            .collect())
    }

    pub async fn metric(&self, street: Street) -> Result<Metric, PgError> {
        let street = street as i8;
        const SQL_METRIC: &'static str = r#"
            SELECT 
                a1.abs ^ a2.abs AS xor,
                dx 
            FROM abstraction a1 
            JOIN abstraction a2 
            ON a1.st = a2.st 
            WHERE a1.st = $1 AND a1.abs != a2.abs;
        "#;
        Ok(self
            .0
            .query(SQL_METRIC, &[&street])
            .await?
            .iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, f32>(1)))
            .map(|(xor, distance)| (Pair::from(xor), distance))
            .collect::<BTreeMap<Pair, f32>>()
            .pipe(Metric::from))
    }

    pub async fn abs_distance(&self, x: Observation, y: Observation) -> Result<Energy, PgError> {
        if x.street() != y.street() {
            return Err(PgError::__private_api_timeout());
        }
        if x == y {
            return Ok(0 as Energy);
        }
        let x = i64::from(x);
        let y = i64::from(y);
        const SQL_ABS_DISTANCE: &'static str = r#"
            SELECT 
                (e1.abs ^ e2.abs) AS xor, 
                m.dx              AS dx
            FROM encoder e1
            JOIN encoder e2
                ON  e1.obs = $1
                AND e2.obs = $2
            JOIN metric m 
                ON (e1.abs ^ e2.abs) = m.xor;
        "#;
        Ok(self
            .0
            .query_one(SQL_ABS_DISTANCE, &[&x, &y])
            .await?
            .get::<_, Energy>(1))
    }

    pub async fn obs_distance(&self, x: Observation, y: Observation) -> Result<Energy, PgError> {
        if x.street() != y.street() {
            return Err(PgError::__private_api_timeout());
        }
        let (ref hx, ref hy, ref metric) = tokio::try_join!(
            self.histogram(x),
            self.histogram(y),
            self.metric(x.street())
        )?;
        Ok(Sinkhorn::from((hx, hy, metric)).minimize().cost())
    }

    pub async fn upload(&self) -> Result<(), PgError> {
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

    #[rustfmt::skip]
    async fn nuke(&self) -> Result<u64, PgError> {
        Ok(self.0.execute(r#"
            DROP SCHEMA public CASCADE;
            CREATE SCHEMA public;
        "#, &[]).await?)
    }
    #[rustfmt::skip]
    async fn recreate(&self) -> Result<u64, PgError> {
        Ok(self.0.execute(r#"
            CREATE TABLE IF NOT EXISTS encoder     (obs  BIGINT, abs  BIGINT);
            CREATE TABLE IF NOT EXISTS metric      (xor  BIGINT, dx   REAL);
            CREATE TABLE IF NOT EXISTS abstraction (abs  BIGINT, st   SMALLINT);
            CREATE TABLE IF NOT EXISTS blueprint   (edge BIGINT, past BIGINT, present BIGINT, future BIGINT, policy REAL, regret REAL);
        "#, &[]).await?)
    }
    #[rustfmt::skip]
    async fn truncate(&self) -> Result<u64, PgError> {
        Ok(self.0.execute(r#"
            TRUNCATE TABLE encoder;
            TRUNCATE TABLE metric;
            TRUNCATE TABLE abstraction;
            TRUNCATE TABLE blueprint;
        "#, &[]).await?)
    }
    #[rustfmt::skip]
    async fn unlogged(&self) -> Result<u64, PgError> {
        Ok(self.0.execute(r#"
            ALTER TABLE encoder      SET UNLOGGED;
            ALTER TABLE metric       SET UNLOGGED;
            ALTER TABLE abstraction  SET UNLOGGED;
            ALTER TABLE blueprint    SET UNLOGGED;
        "#, &[]).await?)
    }
    #[rustfmt::skip]
    async fn copy_blueprint(&self) -> Result<u64, PgError> {
        Ok(self.0.execute(r#"
            COPY blueprint (past, present, future, edge, policy, regret) FROM '/Users/krukah/Code/robopoker/blueprint.profile.pgcopy' WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_blueprint_bucket  ON blueprint (present, past, future);
            CREATE INDEX IF NOT EXISTS idx_blueprint_future  ON blueprint (future);
            CREATE INDEX IF NOT EXISTS idx_blueprint_present ON blueprint (present);
            CREATE INDEX IF NOT EXISTS idx_blueprint_edge    ON blueprint (edge);
            CREATE INDEX IF NOT EXISTS idx_blueprint_past    ON blueprint (past);
        "#, &[]).await?)
    }
    #[rustfmt::skip]
    async fn copy_metric(&self) -> Result<u64, PgError> {
        Ok(self.0.execute(r#"
            COPY metric (xor, dx) FROM '/Users/krukah/Code/robopoker/turn.metric.pgcopy'       WITH (FORMAT BINARY);
            COPY metric (xor, dx) FROM '/Users/krukah/Code/robopoker/flop.metric.pgcopy'       WITH (FORMAT BINARY);
            COPY metric (xor, dx) FROM '/Users/krukah/Code/robopoker/preflop.metric.pgcopy'    WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_metric_xor  ON metric (xor);
            CREATE INDEX IF NOT EXISTS idx_metric_dx   ON metric (dx);
        "#, &[]).await?)
    }
    #[rustfmt::skip]
    async fn copy_encoder(&self) -> Result<u64, PgError> {
        Ok(self.0.execute(r#"
            COPY encoder (obs, abs) FROM '/Users/krukah/Code/robopoker/river.encoder.pgcopy'   WITH (FORMAT BINARY);
            COPY encoder (obs, abs) FROM '/Users/krukah/Code/robopoker/turn.encoder.pgcopy'    WITH (FORMAT BINARY);
            COPY encoder (obs, abs) FROM '/Users/krukah/Code/robopoker/flop.encoder.pgcopy'    WITH (FORMAT BINARY);
            COPY encoder (obs, abs) FROM '/Users/krukah/Code/robopoker/preflop.encoder.pgcopy' WITH (FORMAT BINARY);
            CREATE INDEX IF NOT EXISTS idx_encoder_obs ON encoder (obs);
            CREATE INDEX IF NOT EXISTS idx_encoder_abs ON encoder (abs);
        "#, &[]).await?)
    }
    #[rustfmt::skip]
    async fn copy_abstraction(&self) -> Result<u64, PgError> {
        Ok(self.0.execute(r#"
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
        "#, &[]).await?)
    }
}

const SQL_CLUSTERS: &'static str = r#"
    SELECT 
        e.abs        AS abs,
        a.st         AS street, 
        COUNT(*)     AS n_obs 
    FROM 
        encoder e 
    JOIN 
        abstraction a ON e.abs = a.abs 
    GROUP BY 
        e.abs, a.st 
    ORDER BY 
        a.st, COUNT(*);
"#;
const SQL_HEATMAP: &'static str = r#"
    WITH stabs AS (
        SELECT  abs
        FROM    abstraction
        WHERE   st = 1
    ),
    pairs AS (
        SELECT 
            a.abs                   AS abs1,
            b.abs                   AS abs2,
            (a.abs # b.abs)::bigint AS pxor
        FROM        stabs a
        CROSS JOIN  stabs b
        WHERE       a.abs > b.abs
    )
    SELECT 
        c.abs1,
        c.abs2,
        COALESCE(m.dx, 0) AS dst
    FROM pairs c
    LEFT JOIN metric m ON m.xor = c.pxor 
"#;
