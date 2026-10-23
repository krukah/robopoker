use super::*;
use crate::*;
use deuce::*;
use serde::{Deserialize, Serialize};

/// One stored hand, in the shape the client can actually reopen.
///
/// It used to ship the rendered form — a board joined into a string, the plays as `{ seq,
/// action, street }` triples of text — while the server reconstructed a `Witness` to produce
/// them and then dropped it.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRecap {
    pub id: uuid::Uuid,
    pub pot: i16,
    pub dealer: usize,
    /// The line, from the reviewing seat's point of view.
    pub recall: Witness,
    /// What each seat held, where the table knows — `Some` once a seat has shown, `None` where
    /// it mucked or never had to say. One slot per seat, so "we may not know" and "they did not
    /// reveal" are the same absence rather than two flags to keep in agreement.
    ///
    /// The reviewing seat's own cards ride inside `recall`, exactly as they do in a live
    /// `Snapshot`, so both sources read the same way.
    pub reveals: Vec<Option<Hole>>,
    pub players: Vec<ApiPlayer>,
}

impl ApiRecap {
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn pot(&self) -> i16 {
        self.pot
    }

    pub fn dealer(&self) -> usize {
        self.dealer
    }

    pub fn recall(&self) -> &Witness {
        &self.recall
    }

    pub fn reveals(&self) -> &[Option<Hole>] {
        &self.reveals
    }

    pub fn players(&self) -> &[ApiPlayer] {
        &self.players
    }
}
