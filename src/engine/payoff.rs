use super::player::RoboPlayer;

pub struct Payoff {
    pub winner: &'static RoboPlayer,
    pub winnings: u32,
}
