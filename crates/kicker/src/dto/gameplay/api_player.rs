use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiPlayer {
    pub seat: usize,
    pub stack: i16,
    pub won: i16,
}

impl ApiPlayer {
    pub fn seat(&self) -> usize {
        self.seat
    }

    pub fn stack(&self) -> i16 {
        self.stack
    }

    pub fn won(&self) -> i16 {
        self.won
    }
}
