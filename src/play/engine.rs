#![allow(dead_code)]

pub struct Table {
    n_hands: u32,
    // hand: History,
}

impl Table {
    pub fn new() -> Self {
        Table { n_hands: 0 }
    }
}
