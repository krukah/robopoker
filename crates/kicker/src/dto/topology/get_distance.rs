use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetDistance {
    pub a: String,
    pub b: String,
}

impl GetDistance {
    pub fn a(&self) -> &str {
        &self.a
    }

    pub fn b(&self) -> &str {
        &self.b
    }
}
