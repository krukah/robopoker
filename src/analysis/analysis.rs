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
}
