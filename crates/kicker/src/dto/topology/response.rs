use crate::*;
use deuce::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSample {
    pub obs: Observation,
    pub abs: Abstraction,
    pub equity: f32,
    pub density: f32,
    pub distance: f32,
}

impl ApiSample {
    pub fn obs(&self) -> Observation {
        self.obs
    }

    pub fn abs(&self) -> Abstraction {
        self.abs
    }

    pub fn equity(&self) -> f32 {
        self.equity
    }

    pub fn density(&self) -> f32 {
        self.density
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }
}

/// Bridges a `SELECT obs, abs, equity, density[, distance]` row straight to
/// the DTO via the domain codecs (`Isomorphism`/`Abstraction` FromSql).
#[cfg(feature = "sql")]
impl From<tokio_postgres::Row> for ApiSample {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            obs: Observation::from(row.get::<_, Isomorphism>("obs")),
            abs: row.get::<_, Abstraction>("abs"),
            equity: row.get::<_, f32>("equity"),
            density: row.get::<_, f32>("density"),
            distance: row.try_get::<_, f32>("distance").unwrap_or_default(),
        }
    }
}
