use super::player::Player;

pub struct Payoff {
    pub winner: &'static Player,
    pub winnings: u32,
}
