use super::*;
use rbp_cards::*;
use rbp_core::*;

/// Per-seat authoritative view of the room state at a moment in time.
///
/// The server constructs a `Snapshot` for each connected player after every
/// state change and ships it over the wire. The client renders directly from
/// the latest snapshot — no incremental state machine, no event reconciliation.
///
/// `witness` is the single source of truth for everything derivable from the
/// hand's action history (board, current `Game`, legal moves, who's to act).
/// The other fields carry only what `witness` cannot represent on its own:
/// villain reveals at showdown, settlements, session history, lifecycle phase.
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

/// A settled hand. Per-seat PnL derives from `settlements`; per-seat
/// hand-replay derives from each seat's `Witness` (only built on demand
/// for the player who's viewing).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompletedHand {
    pub epoch: u64,
    pub settlements: Vec<Settlement>,
}

impl CompletedHand {
    pub fn pnl(&self, seat: Position) -> Chips {
        self.settlements.get(seat).map_or(0, |s| s.pnl().won())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Turn;

    fn empty_snap() -> Snapshot {
        Snapshot {
            hand: 0,
            phase: Phase::Waiting,
            witness: Witness::initial(Turn::Choice(0)),
            reveals: vec![None, None],
            settlements: vec![],
            history: vec![],
        }
    }

    #[test]
    fn snapshot_roundtrips_through_json() {
        let snap = empty_snap();
        let json = serde_json::to_string(&snap).expect("serialize");
        let back: Snapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(snap.hand, back.hand);
        assert_eq!(snap.phase, back.phase);
    }

    #[test]
    fn server_message_snapshot_tag() {
        let json = ServerMessage::Snapshot(empty_snap()).to_json();
        assert!(json.contains(r#""type":"snapshot""#));
    }

    #[test]
    fn server_message_welcome_tag() {
        let json = ServerMessage::welcome("room-id", 0).to_json();
        assert!(json.contains(r#""type":"welcome""#));
        assert!(json.contains(r#""room":"room-id""#));
        assert!(json.contains(r#""seat":0"#));
    }

    #[test]
    fn server_message_session_end_serializes_reason() {
        let json = ServerMessage::session_end([100, 200], Reason::Busted).to_json();
        assert!(json.contains(r#""type":"session_end""#));
        assert!(json.contains(r#""reason":"busted""#));
    }
}
