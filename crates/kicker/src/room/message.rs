use super::*;
use pokerkit::*;

/// Wire protocol from server to client.
///
/// The hot path is `Snapshot`, a complete per-seat view of the room state.
/// `Welcome` is sent once at connect, so the client knows who it is and how
/// fast its table is paced. `Rejected` answers an illegal action, and
/// `SessionEnd` terminates the session and explains why.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome {
        room: String,
        seat: Position,
        pacing: Pacing,
    },
    Snapshot(Snapshot),
    Rejected {
        reason: String,
        legal: Vec<Action>,
    },
    SessionEnd {
        stacks: [Chips; N],
        reason: Reason,
    },
}

impl ServerMessage {
    /// The pacing rides along because the client draws rings against it.
    /// Deriving it from a default instead is a guess, and it is wrong the
    /// moment a room is paced at anything but the default tempo.
    pub fn welcome(room: &str, seat: Position, pacing: Pacing) -> Self {
        Self::Welcome {
            room: room.to_string(),
            seat,
            pacing,
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
