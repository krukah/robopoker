use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ReplaceAbs {
    pub wrt: Abstraction,
}

impl ReplaceAbs {
    pub fn wrt(&self) -> Abstraction {
        self.wrt
    }
}
