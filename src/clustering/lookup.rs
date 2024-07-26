use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::observation::Observation;
use super::xor::Pair;
use redis::AsyncCommands;

/// Wrapper around sqlx::PgPool. This struct is responsible for all storage interactions.
/// We can swap this out with Redis or a HashMap or BTreeMap.
/// TODO: benchmark different persistence implementations
pub struct PostgresLookup {
    db: sqlx::PgPool,
}

impl PostgresLookup {
    /// Create a new Lookup instance with database connection
    pub async fn new() -> Self {
        const DATABASE_URL: &str = "postgres://postgres:postgrespassword@localhost:5432/robopoker";
        let ref url = std::env::var("DATABASE_URL").unwrap_or_else(|_| String::from(DATABASE_URL));
        let postgres = sqlx::PgPool::connect(url)
            .await
            .expect("database to accept connections");
        Self { db: postgres }
    }

    /// Insert row into cluster table
    pub async fn set_obs(&self, obs: Observation, abs: Abstraction) {
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
        .execute(&self.db)
        .await
        .expect("database insert: cluster");
    }

    /// Insert row into metric table
    pub async fn set_xor(&self, xor: Pair, distance: f32) {
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
        .execute(&self.db)
        .await
        .expect("database insert: metric");
    }

    /// Query Observation -> Abstraction table
    pub async fn get_obs(&self, obs: Observation) -> Abstraction {
        let abs = sqlx::query!(
            r#"
                SELECT abstraction
                FROM cluster
                WHERE observation = $1"#,
            i64::from(obs),
        )
        .fetch_one(&self.db)
        .await
        .expect("to respond to cluster query")
        .abstraction
        .expect("to have computed cluster previously");
        Abstraction::from(abs)
    }

    /// Query Pair -> f32 table
    pub async fn get_xor(&self, xor: Pair) -> f32 {
        let distance = sqlx::query!(
            r#"
                SELECT distance
                FROM metric
                WHERE xor = $1"#,
            i64::from(xor),
        )
        .fetch_one(&self.db)
        .await
        .expect("to respond to metric query")
        .distance
        .expect("to have computed metric previously");
        distance as f32
    }

    /// ~1Kb download
    /// this could possibly be implemented as a join?
    /// fml a big Vec<> of these is gonna have to fit
    /// in memory for the centroid calculation
    pub async fn get_histogram(&self, obs: Observation) -> Histogram {
        let mut abstractions = Vec::new();
        let successors = obs.successors();
        for succ in successors {
            let abstraction = self.get_obs(succ).await;
            abstractions.push(abstraction);
        }
        Histogram::from(abstractions)
    }
}

pub struct RedisLookup {
    client: redis::Client,
}

impl RedisLookup {
    pub async fn new() -> Self {
        const REDIS_URL: &str = "redis://localhost:6379";
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| String::from(REDIS_URL));
        let client = redis::Client::open(url).expect("Redis client to connect");
        Self { client }
    }

    pub async fn set_obs(&self, obs: Observation, abs: Abstraction) {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection");
        let key = format!("cluster:{}", i64::from(obs));
        conn.set::<String, i64, redis::Value>(key, i64::from(abs))
            .await
            .expect("Redis set: cluster");
    }

    pub async fn set_xor(&self, xor: Pair, distance: f32) {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection");
        let key = format!("metric:{}", i64::from(xor));
        conn.set::<String, f32, redis::Value>(key, distance)
            .await
            .expect("Redis set: metric");
    }

    pub async fn get_obs(&self, obs: Observation) -> Abstraction {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection");
        let key = format!("cluster:{}", i64::from(obs));
        let abs: i64 = conn.get(key).await.expect("Redis get: cluster");
        Abstraction::from(abs)
    }

    pub async fn get_xor(&self, xor: Pair) -> f32 {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection");
        let key = format!("metric:{}", i64::from(xor));
        let distance: String = conn.get(key).await.expect("Redis get: metric");
        distance.parse().expect("Valid f32")
    }

    pub async fn get_histogram(&self, obs: Observation) -> Histogram {
        let mut abstractions = Vec::new();
        let successors = obs.successors();
        for succ in successors {
            let abstraction = self.get_obs(succ).await;
            abstractions.push(abstraction);
        }
        Histogram::from(abstractions)
    }
}

trait Lookup {
    async fn set_obs(&self, obs: Observation, abs: Abstraction);
    async fn set_xor(&self, xor: Pair, distance: f32);
    async fn get_obs(&self, obs: Observation) -> Abstraction;
    async fn get_xor(&self, xor: Pair) -> f32;
    async fn get_histogram(&self, obs: Observation) -> Histogram;
}

impl Lookup for PostgresLookup {
    async fn set_obs(&self, obs: Observation, abs: Abstraction) {
        PostgresLookup::set_obs(self, obs, abs).await;
    }

    async fn set_xor(&self, xor: Pair, distance: f32) {
        PostgresLookup::set_xor(self, xor, distance).await;
    }

    async fn get_obs(&self, obs: Observation) -> Abstraction {
        PostgresLookup::get_obs(self, obs).await
    }

    async fn get_xor(&self, xor: Pair) -> f32 {
        PostgresLookup::get_xor(self, xor).await
    }

    async fn get_histogram(&self, obs: Observation) -> Histogram {
        PostgresLookup::get_histogram(self, obs).await
    }
}

impl Lookup for RedisLookup {
    async fn set_obs(&self, obs: Observation, abs: Abstraction) {
        RedisLookup::set_obs(self, obs, abs).await;
    }

    async fn set_xor(&self, xor: Pair, distance: f32) {
        RedisLookup::set_xor(self, xor, distance).await;
    }

    async fn get_obs(&self, obs: Observation) -> Abstraction {
        RedisLookup::get_obs(self, obs).await
    }

    async fn get_xor(&self, xor: Pair) -> f32 {
        RedisLookup::get_xor(self, xor).await
    }

    async fn get_histogram(&self, obs: Observation) -> Histogram {
        RedisLookup::get_histogram(self, obs).await
    }
}
