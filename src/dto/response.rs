use crate::mccfr::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSample {
    pub obs: String,
    pub abs: String,
    pub equity: f32,
    pub density: f32,
    pub distance: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDecision {
    pub edge: String,
    pub mass: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStrategy {
    pub history: i64,
    pub present: i64,
    pub choices: i64,
    pub accumulated: BTreeMap<String, f32>,
}

impl From<Strategy> for ApiStrategy {
    fn from(strategy: Strategy) -> Self {
        let (history, present, choices) = (*strategy.info()).into();
        Self {
            history: history.into(),
            present: present.into(),
            choices: choices.into(),
            accumulated: strategy
                .accumulated()
                .into_iter()
                .map(|(edge, policy)| (edge.to_string(), *policy))
                .collect(),
        }
    }
}

impl From<Decision> for ApiDecision {
    fn from(decision: Decision) -> Self {
        Self {
            edge: decision.edge.to_string(),
            mass: decision.mass,
        }
    }
}

#[cfg(feature = "database")]
use crate::cards::*;
#[cfg(feature = "database")]
use crate::gameplay::*;
#[cfg(feature = "database")]
use crate::*;

#[cfg(feature = "database")]
impl From<tokio_postgres::Row> for ApiSample {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            obs: Observation::from(row.get::<_, i64>("obs")).to_string(),
            abs: Abstraction::from(row.get::<_, i64>("abs")).to_string(),
            equity: row.get::<_, f32>("equity").into(),
            density: row.get::<_, f32>("density").into(),
            distance: row.try_get::<_, f32>("distance").unwrap_or_default().into(),
        }
    }
}

#[cfg(feature = "database")]
impl From<tokio_postgres::Row> for Decision {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            edge: Edge::from(row.get::<_, i64>("edge") as u64),
            mass: Probability::from(row.get::<_, f32>("policy")),
        }
    }
}
