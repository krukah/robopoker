pub struct Robot;

impl Debug for Robot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player")
    }
}

use std::fmt::Debug;
