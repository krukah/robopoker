use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use crate::mccfr::edge::Edge;
use crate::Probability;
use serde::Serialize;

#[derive(Serialize)]
pub struct Sample {
    pub obs: String,
    pub abs: String,
    pub equity: f32,
    pub density: f32,
    pub distance: f32,
}

#[derive(Serialize)]
pub struct Decision {
    pub edge: String,
    pub prob: Probability,
}

impl From<tokio_postgres::Row> for Sample {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            obs: Observation::from(row.get::<_, i64>(0)).equivalent(),
            abs: Abstraction::from(row.get::<_, i64>(1)).to_string(),
            equity: row.get::<_, f32>(2).into(),
            density: row.get::<_, f32>(3).into(),
            distance: row.get::<_, f32>(4).into(),
        }
    }
}

impl From<tokio_postgres::Row> for Decision {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            edge: Edge::from(row.get::<_, i64>("edge") as u64).to_string(),
            prob: Probability::from(row.get::<_, f32>("policy")),
        }
    }
}
