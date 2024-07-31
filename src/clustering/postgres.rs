use crate::clustering::abstraction::Abstraction;
use crate::clustering::histogram::Histogram;
use crate::clustering::observation::Observation;
use crate::clustering::xor::Pair;

#[derive(Clone)]
pub struct PostgresLookup {
    pool: sqlx::PgPool,
}

impl PostgresLookup {
    /// Create a new Lookup instance with database connection
    pub async fn new() -> Self {
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
    pub async fn set_obs(&mut self, obs: Observation, abs: Abstraction) {
        sqlx::query(
            r#"
                INSERT INTO cluster (observation, abstraction, street)
                VALUES              ($1, $2, $3)
                ON CONFLICT         (observation)
                DO UPDATE SET       abstraction = $2
            "#,
        )
        .bind(i64::from(obs))
        .bind(i64::from(abs))
        .bind(obs.street() as i16)
        .execute(&self.pool)
        .await
        .expect("database insert: cluster");
    }

    /// Insert row into metric table
    pub async fn set_xor(&mut self, xor: Pair, distance: f32) {
        sqlx::query(
            r#"
                INSERT INTO metric  (xor, distance, street)
                VALUES              ($1, $2, $3)
                ON CONFLICT         (xor)
                DO UPDATE SET       distance = $2
            "#,
        )
        .bind(i64::from(xor))
        .bind(f32::from(distance))
        .bind(0) // TODO: deprecate Street column from schema
        .execute(&self.pool)
        .await
        .expect("database insert: metric");
    }

    /// Query Observation -> Abstraction table
    pub async fn get_obs(&self, obs: Observation) -> Abstraction {
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
    pub async fn get_xor(&self, xor: Pair) -> f32 {
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
    pub async fn set_obs_batch(&mut self, batch: Vec<(Observation, Abstraction)>) {
        sqlx::QueryBuilder::new(
            r#"
                INSERT INTO cluster
                (street, observation, abstraction)
            "#,
        )
        .push_values(batch, |mut list, (obs, abs)| {
            list.push_bind(obs.street() as i16)
                .push_bind(i64::from(obs.clone()))
                .push_bind(i64::from(abs.clone()));
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
    pub async fn set_xor_batch(&mut self, batch: Vec<(Pair, f32)>) {
        sqlx::QueryBuilder::new(
            r#"
                INSERT INTO metric
                (street, xor, distance)
            "#,
        )
        .push_values(batch, |mut list, (xor, distance)| {
            list.push_bind(0)
                .push_bind(i64::from(xor.clone()))
                .push_bind(f32::from(distance.clone())); // TODO: deprecate Street column from schema
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

    /// ~1Kb download
    /// this could possibly be implemented as a join?
    /// fml a big Vec<> of these is gonna have to fit
    /// in memory for the centroid calculation
    pub async fn get_histogram(&self, obs: Observation) -> Histogram {
        let mut abstractions = Vec::new();
        let successors = obs.outnodes();
        for succ in successors {
            let abstraction = self.get_obs(succ).await;
            abstractions.push(abstraction);
        }
        Histogram::from(abstractions)
    }
}
