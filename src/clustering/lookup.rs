use crate::cards::street::Street;

use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::observation::Observation;
use super::xor::Pair;

/// Wrapper around sqlx::PgPool. This struct is responsible for all storage interactions.
/// We can swap this out with Redis or a HashMap or BTreeMap.
/// TODO: benchmark different persistence implementations
pub struct Lookup {
    db: sqlx::PgPool,
}

impl From<sqlx::PgPool> for Lookup {
    fn from(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

impl Lookup {
    /// Insert row into cluster table
    pub async fn set_obs(&self, st: Street, obs: Observation, abs: Abstraction) {
        sqlx::query(
            r#"
                INSERT INTO cluster (observation, abstraction, street)
                VALUES              ($1, $2, $3)
                ON CONFLICT         (observation)
                DO UPDATE SET       abstraction = $2"#,
        )
        .bind(i64::from(obs))
        .bind(i64::from(abs))
        .bind(st as i64)
        .execute(&self.db)
        .await
        .expect("database insert: cluster");
    }

    /// Insert row into metric table
    pub async fn set_xor(&self, st: Street, xor: Pair, distance: f32) {
        sqlx::query(
            r#"
                INSERT INTO metric  (xor, distance, street)
                VALUES              ($1, $2, $3)
                ON CONFLICT         (xor)
                DO UPDATE SET       distance = $2"#,
        )
        .bind(i64::from(xor))
        .bind(f32::from(distance))
        .bind(st as i64)
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
