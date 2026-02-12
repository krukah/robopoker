use super::*;
use rbp_gameplay::*;

/// Errors that can occur during protocol operations.
#[derive(Debug, Clone)]
pub enum ProtocolError {
    InvalidAction(String),
    IllegalAction(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAction(s) => write!(f, "invalid action: {}", s),
            Self::IllegalAction(s) => write!(f, "illegal action: {}", s),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Handles Event to ServerMessage conversion and action parsing.
/// Centralizes the protocol layer between internal events and wire format.
pub struct Protocol;

impl Protocol {
    /// Converts an internal Event to a wire ServerMessage.
    /// Returns None for events that should not be sent to clients (e.g., Disconnect).
    pub fn encode(event: &Event) -> Option<ServerMessage> {
        match event {
            Event::HandStart {
                hand,
                dealer,
                stacks,
            } => Some(ServerMessage::hand_start(*hand, *dealer, stacks.clone())),
            Event::HoleCards { hand, hole } => Some(ServerMessage::hole_cards(*hand, *hole)),
            Event::Board {
                hand,
                street,
                board,
            } => Some(ServerMessage::board(*hand, *street, *board)),
            Event::Action {
                hand,
                seat,
                action,
                pot,
            } => Some(ServerMessage::action(
                *hand,
                *seat,
                &action.to_string(),
                *pot,
            )),
            Event::Decision { hand, recall } => Some(ServerMessage::decision(
                *hand,
                recall
                    .head()
                    .legal()
                    .iter()
                    .map(|a| a.to_string())
                    .collect(),
                recall.board().iter().map(|c| c.to_string()).collect(),
                recall.head().pot(),
            )),
            Event::Reveal { hand, seat, hole } => Some(ServerMessage::showdown(
                *hand,
                vec![Reveal {
                    seat: *seat,
                    cards: hole.map(|h| h.to_string()),
                }],
            )),
            Event::HandEnd { hand, winners } => Some(ServerMessage::hand_end(
                *hand,
                winners
                    .iter()
                    .map(|(seat, amount)| Winner {
                        seat: *seat,
                        amount: *amount,
                    })
                    .collect(),
            )),
            Event::Disconnect(_) => None,
        }
    }
    /// Parses a client message string into an Action.
    pub fn decode(s: &str) -> Result<Action, ProtocolError> {
        Action::try_from(s).map_err(|_| ProtocolError::InvalidAction(s.to_string()))
    }
    /// Validates an action is legal given the available actions.
    pub fn validate(action: Action, legal: &[Action]) -> Result<Action, ProtocolError> {
        legal
            .contains(&action)
            .then_some(action)
            .ok_or_else(|| ProtocolError::IllegalAction(action.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decode_valid_action() {
        assert!(Protocol::decode("fold").is_ok());
        assert!(Protocol::decode("check").is_ok());
        assert!(Protocol::decode("call 10").is_ok());
        assert!(Protocol::decode("raise 20").is_ok());
    }
    #[test]
    fn decode_invalid_action() {
        assert!(Protocol::decode("invalid").is_err());
        assert!(Protocol::decode("call").is_err()); // missing amount
    }
    #[test]
    fn validate_legal_action() {
        let legal = vec![
            Action::Fold, //
            Action::Check,
        ];
        assert!(Protocol::validate(Action::Fold, &legal).is_ok());
        assert!(Protocol::validate(Action::Check, &legal).is_ok());
    }
    #[test]
    fn validate_illegal_action() {
        let legal = vec![
            Action::Fold, //
            Action::Check,
        ];
        assert!(Protocol::validate(Action::Raise(10), &legal).is_err());
    }
}
