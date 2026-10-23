use deuce::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SetStreets {
    pub street: Street,
}

impl SetStreets {
    pub fn street(&self) -> Street {
        self.street
    }
}
