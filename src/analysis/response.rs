use serde::Serialize;

#[derive(Serialize)]
pub struct ObsAbsResponse {
    pub obs: String,
    pub abs: String,
    pub equity: f32,
    pub density: f32,
    pub distance: f32,
}
