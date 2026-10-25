/// Player betting status within a hand. `Shoving` is all-in — no more
/// decisions, but still contesting the pot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Betting,
    Shoving,
    Folding,
}
impl State {
    /// True if player is still competing for the pot.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Betting | Self::Shoving)
    }
}
impl TryFrom<&str> for State {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "P" => Ok(State::Betting),
            "S" => Ok(State::Shoving),
            "F" => Ok(State::Folding),
            _ => Err(anyhow::anyhow!("invalid state string")),
        }
    }
}
impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Betting => write!(f, "P"),
            State::Shoving => write!(f, "S"),
            State::Folding => write!(f, "F"),
        }
    }
}
