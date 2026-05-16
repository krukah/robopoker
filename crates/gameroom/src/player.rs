use super::event::*;
use rbp_gameplay::*;
use std::time::Duration;

/// Trait for entities that make poker decisions.
/// Implementations can be CPU players, humans via CLI, network players via WebSocket, etc.
///
/// The async design allows:
/// - CPU players to spawn blocking computation in separate threads
/// - Human players to await user input without blocking the room
/// - Network players to await remote responses with timeouts
///
/// Participant is transport-agnostic: the Room doesn't care whether
/// decisions come from local computation, stdin, HTTP, WebSocket, etc.
#[async_trait::async_trait]
pub trait Player: Send {
    /// Check if the player is still connected.
    /// Returns false when the player has disconnected (channel closed, etc.).
    /// Default implementation returns true (CPU players are always alive).
    fn alive(&self) -> bool {
        true
    }
    /// Whether this player voluntarily shows cards at showdown.
    /// Default is false (mucks when allowed). Override to true for bots.
    fn shows(&self) -> bool {
        false
    }
    /// Post-decision delay for pacing. CPU players override this to simulate
    /// thinking time; human/network players return ZERO (their latency IS the delay).
    fn pace(&self) -> Duration {
        Duration::ZERO
    }
    /// Make a decision given complete game state.
    /// Called when it's this player's turn to act.
    /// Recall contains all information visible to this player.
    async fn decide(&mut self, recall: &Witness) -> Action;
    /// Receive notification of game events.
    /// Called for all public actions and private events relevant to this player.
    /// Useful for updating UI, logging, or maintaining local state.
    /// Not required for decision-making (Witness is self-contained).
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
        (**self).notify(event).await
    }
}
