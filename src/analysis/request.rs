use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReplaceObsRequest {
    pub obs: String,
}

#[derive(Deserialize)]
pub struct ReplaceAbsRequest {
    pub wrt: String,
}

#[derive(Deserialize)]
pub struct ObsAbsWrtRequest {
    pub wrt: String,
    pub obs: String,
}
