use serde::Deserialize;

#[derive(Deserialize)]
pub struct SetStreets {
    pub street: String, // "P"
}

#[derive(Deserialize)]
pub struct ReplaceObs {
    pub obs: String, // "2c 3c ~ 4c 5c 6c"
}

#[derive(Deserialize)]
pub struct RowWrtObs {
    pub obs: String, // "2c 3c ~ 4c 5c 6c"
}

#[derive(Deserialize)]
pub struct ReplaceAbs {
    pub wrt: String, // "P::22"
}

#[derive(Deserialize)]
pub struct ReplaceRow {
    pub wrt: String, // "P::22"
    pub obs: String, // "2c 3c ~ 4c 5c 6c"
}

#[derive(Deserialize)]
pub struct ReplaceOne {
    pub wrt: String, // "P::22"
    pub abs: String, // "P::22"
}

#[derive(Deserialize)]
pub struct ReplaceAll {
    pub wrt: String,            // "P::22"
    pub neighbors: Vec<String>, // ["2c 3c", "Ad Kh"]
}

#[derive(Deserialize)]
pub struct ObsHist {
    pub obs: String, // "2c 3c"
}

#[derive(Deserialize)]
pub struct AbsHist {
    pub abs: String, // "P::22"
}

#[derive(Deserialize)]
pub struct GetPolicy {
    pub turn: String,      // "P0"
    pub seen: String,      // "2c 3c ~ 4c 5c 6c"
    pub past: Vec<String>, // ["BLIND 1", "BLIND 2", "RAISE 5", "CALL 4", "DRAW Kc Kd Ks"]
}
