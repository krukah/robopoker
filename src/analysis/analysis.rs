use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use std::sync::Arc;
use tokio_postgres::Client;
use tokio_postgres::Error as PgError;

pub struct Analysis(Arc<Client>);

impl Analysis {
    pub fn new(client: Client) -> Self {
        Self(Arc::new(client))
    }
    pub async fn cluster(&self, obs: Observation) -> Result<Abstraction, PgError> {
        unimplemented!()
    }
    pub async fn neighbors(&self, abs: Abstraction) -> Result<Vec<Abstraction>, PgError> {
        unimplemented!()
    }
    pub async fn constituents(&self, abs: Abstraction) -> Result<Vec<Observation>, PgError> {
        unimplemented!()
    }
    pub async fn abs_distance(&self, x: Observation, y: Observation) -> Result<f32, PgError> {
        unimplemented!()
    }
    pub async fn obs_distance(&self, x: Observation, y: Observation) -> Result<f32, PgError> {
        unimplemented!()
    }
    pub async fn upload(&self) -> Result<(), PgError> {
        self.0.execute(SQL_UPLOAD, &[]).await.map(std::mem::drop)
    }
}

const SQL_UPLOAD: &'static str = r#"
    -- create tables
    CREATE TABLE IF NOT EXISTS encoder     (obs  BIGINT, abs  BIGINT);
    CREATE TABLE IF NOT EXISTS metric      (xor  BIGINT, dx   REAL);
    CREATE TABLE IF NOT EXISTS abstraction (abs  BIGINT, st   SMALLINT);
    CREATE TABLE IF NOT EXISTS blueprint   (edge BIGINT, past BIGINT, present BIGINT, future BIGINT, policy REAL, regret REAL);
    
    -- truncate for copy insert performance and idempotence
    TRUNCATE TABLE encoder;
    TRUNCATE TABLE metric;
    TRUNCATE TABLE abstraction;
    TRUNCATE TABLE blueprint;
    
    -- set unlogged for copy insert performance
    ALTER TABLE encoder      SET UNLOGGED;
    ALTER TABLE metric       SET UNLOGGED;
    ALTER TABLE abstraction  SET UNLOGGED;
    ALTER TABLE blueprint    SET UNLOGGED;
    
    -- blueprint --
    -- add indices for convenient joins
    COPY blueprint (past, present, future, edge, policy, regret) FROM 'blueprint.pgcopy' WITH (FORMAT BINARY);
    CREATE INDEX IF NOT EXISTS idx_blueprint_bucket  ON blueprint (present, past, future);
    CREATE INDEX IF NOT EXISTS idx_blueprint_future  ON blueprint (future);
    CREATE INDEX IF NOT EXISTS idx_blueprint_present ON blueprint (present);
    CREATE INDEX IF NOT EXISTS idx_blueprint_edge    ON blueprint (edge);
    CREATE INDEX IF NOT EXISTS idx_blueprint_past    ON blueprint (past);
    
    -- metric --
    -- (skips river)
    COPY metric (xor, dx) FROM 'turn.metric.pgcopy'     WITH (FORMAT BINARY);
    COPY metric (xor, dx) FROM 'flop.metric.pgcopy'     WITH (FORMAT BINARY);
    COPY metric (xor, dx) FROM 'preflop.metric.pgcopy'  WITH (FORMAT BINARY);
    CREATE INDEX IF NOT EXISTS idx_metric_xor  ON metric (xor);
    CREATE INDEX IF NOT EXISTS idx_metric_dx   ON metric (dx);
    
    -- encoder --
    -- (skips preflop)
    COPY encoder (obs, abs) FROM 'river.encoder.pgcopy' WITH (FORMAT BINARY);
    COPY encoder (obs, abs) FROM 'turn.encoder.pgcopy'  WITH (FORMAT BINARY);
    COPY encoder (obs, abs) FROM 'flop.encoder.pgcopy'  WITH (FORMAT BINARY);
    CREATE INDEX IF NOT EXISTS idx_encoder_obs ON encoder (obs);
    CREATE INDEX IF NOT EXISTS idx_encoder_abs ON encoder (abs);
    
    -- abstraction --
    -- map distinct encoder abs -> obs -> street
    CREATE OR REPLACE FUNCTION street(obs BIGINT) RETURNS SMALLINT AS
    $$
    DECLARE
        obits   BIT(64);
        n_cards INTEGER := 0;
        i       INTEGER;
    BEGIN
        obits := obs::BIT(64);
        FOR i IN 0..7 LOOP
            IF   obits[(i * 8 + 1):(i * 8 + 8)] <> B'00000000'
            THEN n_cards := n_cards + 1;
            END IF;
        END LOOP;
        IF    n_cards = 2 THEN RETURN 0;  -- Street::Pref
        ELSIF n_cards = 5 THEN RETURN 1;  -- Street::Flop
        ELSIF n_cards = 6 THEN RETURN 2;  -- Street::Turn
        ELSIF n_cards = 7 THEN RETURN 3;  -- Street::River
        ELSE  RAISE EXCEPTION 'Invalid n_cards: %', n_cards;
        END IF;
    END;
    $$
    LANGUAGE
        plpgsql;
    INSERT INTO abstraction (abs, st) AS
    SELECT
        e.abs                AS abs,
        street(MIN(e.obs))   AS st
    FROM encoder e
    GROUP BY e.abs;
    CREATE INDEX IF NOT EXISTS idx_abstraction_abs ON abstraction (abs);
    CREATE INDEX IF NOT EXISTS idx_abstraction_st  ON abstraction (st);
    
    -- set logged now that tables are populated
    ALTER TABLE encoder      SET LOGGED;
    ALTER TABLE metric       SET LOGGED;
    ALTER TABLE abstraction  SET LOGGED;
    ALTER TABLE blueprint    SET LOGGED;
"#;
