use serde::Deserialize;

#[derive(Deserialize)]
pub struct SetStreets {
    pub street: String,
}

#[derive(Deserialize)]
pub struct ReplaceObs {
    pub obs: String,
}

#[derive(Deserialize)]
pub struct RowWrtObs {
    pub obs: String,
}

#[derive(Deserialize)]
pub struct ReplaceAbs {
    pub wrt: String,
}

#[derive(Deserialize)]
pub struct ReplaceRow {
    pub wrt: String,
    pub obs: String,
}

#[derive(Deserialize)]
pub struct ReplaceOne {
    pub wrt: String,
    pub abs: String,
}

#[derive(Deserialize)]
pub struct ReplaceAll {
    pub wrt: String,
    pub neighbors: Vec<String>,
}
