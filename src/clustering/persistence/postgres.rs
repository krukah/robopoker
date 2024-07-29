use super::storage::Storage;
use crate::clustering::abstraction::Abstraction;
use crate::clustering::observation::Observation;
use crate::clustering::xor::Pair;

#[derive(Clone)]
pub struct PostgresLookup {
    pool: sqlx::PgPool,
}

impl Storage for PostgresLookup {
    /// Create a new Lookup instance with database connection
    async fn new() -> Self {
        let ref url = std::env::var("DATABASE_URL").expect("DATABASE_URL in environment");
        let pool = sqlx::PgPool::connect(url)
            .await
            .expect("database to accept connections");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrations to run");
        Self { pool }
    }

    /// Insert row into cluster table
    async fn set_obs(&mut self, obs: Observation, abs: Abstraction) {
        sqlx::query(
            r#"
                INSERT INTO cluster (observation, abstraction, street)
                VALUES              ($1, $2, $3)
                ON CONFLICT         (observation)
                DO UPDATE SET       abstraction = $2"#,
        )
        .bind(i64::from(obs))
        .bind(i64::from(abs))
        .bind(0) // TODO: deprecate Street column from schema
        .execute(&self.pool)
        .await
        .expect("database insert: cluster");
    }

    /// Insert row into metric table
    async fn set_xor(&mut self, xor: Pair, distance: f32) {
        sqlx::query(
            r#"
                INSERT INTO metric  (xor, distance, street)
                VALUES              ($1, $2, $3)
                ON CONFLICT         (xor)
                DO UPDATE SET       distance = $2"#,
        )
        .bind(i64::from(xor))
        .bind(f32::from(distance))
        .bind(0) // TODO: deprecate Street column from schema
        .execute(&self.pool)
        .await
        .expect("database insert: metric");
    }

    /// Query Observation -> Abstraction table
    async fn get_obs(&self, obs: Observation) -> Abstraction {
        let query = format!(
            r#"
                SELECT abstraction
                FROM cluster
                WHERE observation = {}
            "#,
            i64::from(obs),
        );
        let hash = sqlx::query_as::<_, (Option<i64>,)>(query.as_str())
            .fetch_one(&self.pool)
            .await
            .expect("to respond to cluster query")
            .0
            .expect("to have computed cluster previously");
        Abstraction::from(hash)
    }

    /// Query Pair -> f32 table
    async fn get_xor(&self, xor: Pair) -> f32 {
        let query = format!(
            r#"
                SELECT distance
                FROM metric
                WHERE xor = {}
            "#,
            i64::from(xor),
        );
        let distance = sqlx::query_as::<_, (Option<f32>,)>(query.as_str())
            .fetch_one(&self.pool)
            .await
            .expect("to respond to metric query")
            .0
            .expect("to have computed metric previously");
        distance
    }

    /// Insert multiple rows into cluster table in batch
    async fn set_obs_batch(&mut self, batch: Vec<(Observation, Abstraction)>) {
        sqlx::QueryBuilder::new(
            r#"
                INSERT INTO cluster
                (observation, abstraction, street)
            "#,
        )
        .push_values(batch, |mut b, (obs, abs)| {
            b.push_bind(i64::from(obs.clone()))
                .push_bind(i64::from(abs.clone()))
                .push_bind(0); // TODO: deprecate Street column from schema
        })
        .push(
            r#"
                ON CONFLICT (observation)
                DO UPDATE
                SET abstraction = EXCLUDED.abstraction
            "#,
        )
        .build()
        .execute(&self.pool)
        .await
        .expect("batch insert cluster");
    }

    /// Insert multiple rows into metric table in batch
    async fn set_xor_batch(&mut self, batch: Vec<(Pair, f32)>) {
        sqlx::QueryBuilder::new(
            r#"
                INSERT INTO metric
                (xor, distance, street)
            "#,
        )
        .push_values(batch, |mut b, (xor, distance)| {
            b.push_bind(i64::from(xor.clone()))
                .push_bind(f32::from(distance.clone()))
                .push_bind(0); // TODO: deprecate Street column from schema
        })
        .push(
            r#"
                ON CONFLICT (xor)
                DO UPDATE
                SET distance = EXCLUDED.distance
            "#,
        )
        .build()
        .execute(&self.pool)
        .await
        .expect("batch insert metric");
    }
}
