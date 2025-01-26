use crate::cards::observation::Observation;
use crate::clustering::abstraction::Abstraction;
use serde::Serialize;
use tokio_postgres::Row;

#[derive(Serialize)]
pub struct Neighbor {
    obs: String,
    abs: String,
    equity: f32,
    density: f32,
    distance: f32,
}

impl From<Row> for Neighbor {
    fn from(row: Row) -> Self {
        Self {
            obs: Observation::from(row.get::<_, i64>(0)).equivalent(),
            abs: Abstraction::from(row.get::<_, i64>(1)).to_string(),
            equity: row.get::<_, f32>(2).into(),
            density: row.get::<_, f32>(3).into(),
            distance: row.get::<_, f32>(4).into(),
        }
    }
}

#[derive(Serialize)]
struct Explorer {
    pub abs: String,
    pub obs: String,
    pub equity: f32,
    pub density: f32,
    pub centrality: f32,
}
