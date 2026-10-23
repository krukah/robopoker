use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AbsHist {
    pub abs: Abstraction,
}

impl AbsHist {
    pub fn abs(&self) -> Abstraction {
        self.abs
    }
}
