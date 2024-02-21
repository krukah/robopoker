use std::cell::RefCell;

use super::player::Player;

pub struct Payoff {
    pub winner: RefCell<Player>,
    pub winnings: u32,
}
