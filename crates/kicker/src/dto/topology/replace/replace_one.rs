use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReplaceOne {
    pub wrt: Abstraction,
    pub abs: Abstraction,
}

impl ReplaceOne {
    pub fn wrt(&self) -> Abstraction {
        self.wrt
    }

    pub fn abs(&self) -> Abstraction {
        self.abs
    }
}
