use super::*;
use fulcrum::*;

/// Why the session ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Busted,
    Left,
    Idle,
}

/// Wire protocol from server to client.
///
/// The hot path is `Snapshot` — a complete per-seat view of the room state.
/// `Welcome` is sent once at connect time so the client knows who it is.
/// `Rejected` is the server's response to an illegal action submission.
/// `SessionEnd` terminates the session and explains why.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome { room: String, seat: Position },
    Snapshot(Snapshot),
    Rejected { reason: String, legal: Vec<Action> },
    SessionEnd { stacks: [Chips; N], reason: Reason },
}

impl ServerMessage {
    pub fn welcome(room: &str, seat: Position) -> Self {
        Self::Welcome {
            room: room.to_string(),
            seat,
        }
    }

    pub fn rejected(reason: &str, legal: Vec<Action>) -> Self {
        Self::Rejected {
            reason: reason.to_string(),
            legal,
        }
    }

    pub fn session_end(stacks: [Chips; N], reason: Reason) -> Self {
        Self::SessionEnd { stacks, reason }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialize server message")
    }
}
