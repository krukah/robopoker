use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SetStreets {
    pub street: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceObs {
    pub obs: String,
}

#[derive(Serialize, Deserialize)]
pub struct RowWrtObs {
    pub obs: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceAbs {
    pub wrt: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceRow {
    pub wrt: String,
    pub obs: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceOne {
    pub wrt: String,
    pub abs: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReplaceAll {
    pub wrt: String,
    pub neighbors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ObsHist {
    pub obs: String,
}

#[derive(Serialize, Deserialize)]
pub struct AbsHist {
    pub abs: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetPolicy {
    pub turn: String,
    pub seen: String,
    pub past: Vec<String>,
}
