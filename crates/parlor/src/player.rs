use super::event::*;
use kicker::*;
use std::time::Duration;

/// Trait for entities that make poker decisions — CPU, human via CLI, network
/// via WebSocket, etc.
///
/// Transport-agnostic: the Room doesn't care where a decision comes from. Async
/// so CPU players can offload blocking work while human and network players
/// await input without stalling the room.
#[async_trait::async_trait]
pub trait Player: Send {
    /// False once the player has disconnected. Defaults to true (CPU players
    /// are always alive).
    fn alive(&self) -> bool {
        true
    }
    /// Whether this player voluntarily shows at showdown. Defaults to false
    /// (mucks when allowed); bots override to true.
    fn shows(&self) -> bool {
        false
    }
    /// Post-decision delay for pacing. CPU players override this to simulate
    /// thinking time; human/network players return ZERO (their latency IS the delay).
    fn pace(&self) -> Duration {
        Duration::ZERO
    }
    /// Decide when it is this player's turn. Recall holds everything visible
    /// to them.
    async fn decide(&mut self, recall: &Witness) -> Action;
    /// Public actions and private events relevant to this player — for UI,
    /// logging, or local state. Not needed to decide; Witness is self-contained.
    async fn notify(&mut self, _: &Event) {}
}

/// Forward `Player` through `Box<dyn Player>` so callers (Casino,
/// slumbot Runtime) can hold heterogeneous compositions in a single
/// boxed slot without bespoke wrappers.
#[async_trait::async_trait]
impl<P: Player + ?Sized> Player for Box<P> {
    fn alive(&self) -> bool {
        (**self).alive()
    }

    fn shows(&self) -> bool {
        (**self).shows()
    }

    fn pace(&self) -> Duration {
        (**self).pace()
    }

    async fn decide(&mut self, recall: &Witness) -> Action {
        (**self).decide(recall).await
    }

    async fn notify(&mut self, event: &Event) {
        (**self).notify(event).await;
    }
}
