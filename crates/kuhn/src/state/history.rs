/// Betting history visible to a player at their decision point.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum History {
    Open,
    Check,
    Bet,
    CheckBet,
}
