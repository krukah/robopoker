pub mod action;
pub mod game;
pub mod payout;
pub mod seat;
pub mod showdown;

pub type Chips = u16;
pub const N: usize = 4;
pub const STACK: Chips = 1_000;
