use super::*;
use deuce::*;

/// Per-seat authoritative view of the room state at a moment in time.
///
/// The server constructs a `Snapshot` for each connected player after every state change and
/// ships it over the wire. The client renders directly from the latest snapshot — no
/// incremental state machine, no event reconciliation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    pub hand: u64,
    pub phase: Phase,
    pub witness: Witness,
    /// Villain hole reveals. Index = seat. Hero's slot is always `None`
    /// (read hero's hole from `witness.seen()`).
    pub reveals: Vec<Option<Hole>>,
    /// Per-seat settlement at hand end. Empty until phase = Settled.
    pub settlements: Vec<Settlement>,
    /// Completed hands in this session, in order.
    pub history: Vec<CompletedHand>,
}

impl Snapshot {
    pub fn hand(&self) -> u64 {
        self.hand
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn witness(&self) -> &Witness {
        &self.witness
    }

    pub fn reveals(&self) -> &[Option<Hole>] {
        &self.reveals
    }

    pub fn settlements(&self) -> &[Settlement] {
        &self.settlements
    }

    pub fn history(&self) -> &[CompletedHand] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Turn;
    use pokerkit::Pacing;

    impl Snapshot {
        fn empty() -> Self {
            Self {
                hand: 0,
                phase: Phase::Waiting,
                witness: Witness::initial(Turn::Choice(0)),
                reveals: vec![None, None],
                settlements: vec![],
                history: vec![],
            }
        }
    }

    #[test]
    fn snapshot_roundtrips_through_json() {
        let snap = Snapshot::empty();
        let json = serde_json::to_string(&snap).expect("serialize");
        let back: Snapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(snap.hand(), back.hand());
        assert_eq!(snap.phase(), back.phase());
    }

    #[test]
    fn server_message_snapshot_tag() {
        let json = ServerMessage::Snapshot(Snapshot::empty()).to_json();
        assert!(json.contains(r#""type":"snapshot""#));
    }

    #[test]
    fn server_message_welcome_tag() {
        let json = ServerMessage::welcome("room-id", 0, Pacing::default()).to_json();
        assert!(json.contains(r#""type":"welcome""#));
        assert!(json.contains(r#""room":"room-id""#));
        assert!(json.contains(r#""seat":0"#));
    }

    #[test]
    fn server_message_session_end_serializes_reason() {
        let json = ServerMessage::session_end([100; pokerkit::N], Reason::Busted).to_json();
        assert!(json.contains(r#""type":"session_end""#));
        assert!(json.contains(r#""reason":"busted""#));
    }
}
