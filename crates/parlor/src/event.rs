use cowboys::*;
use pokerkit::*;

/// How a decision was produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prompt {
    Acted,
    Expired,
}

impl Prompt {
    pub fn expired(self) -> bool {
        self == Self::Expired
    }
}

/// Actor↔engine coordination. State changes flow through `LiveEvent` and the
/// snapshot wire; this enum carries only what the actor and engine need to
/// negotiate a single decision.
#[derive(Clone, Debug)]
pub enum Event {
    /// Engine→actor: it's this player's turn; here is their authoritative recall.
    Decision(Witness),
    /// Actor→engine: the player chose this action.
    Action(Action),
    /// Actor→engine: the player has dropped (channel closed).
    Disconnect(Position),
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Event::Decision(recall) => write!(
                f,
                "Your turn: {}",
                recall
                    .head()
                    .legal()
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Event::Action(action) => write!(f, "{action}"),
            Event::Disconnect(pos) => write!(f, "P{pos}: disconnected"),
        }
    }
}
