use crate::*;
use pokerkit::*;

/// A settled hand. Per-seat PnL derives from `settlements`; per-seat
/// hand-replay derives from each seat's `Witness` (only built on demand
/// for the player who's viewing).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CompletedHand {
    pub epoch: u64,
    pub settlements: Vec<Settlement>,
}

impl CompletedHand {
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn settlements(&self) -> &[Settlement] {
        &self.settlements
    }
}

impl CompletedHand {
    pub fn pnl(&self, seat: Position) -> Chips {
        self.settlements().get(seat).map_or(0, |s| s.pnl().won())
    }
}
