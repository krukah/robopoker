use super::event::*;
use rbp_gameplay::*;

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
    /// Make a decision given complete game state.
    /// Called when it's this player's turn to act.
    /// Recall contains all information visible to this player.
    async fn decide(&mut self, recall: &Partial) -> Action;
    /// Receive notification of game events.
    /// Called for all public actions and private events relevant to this player.
    /// Useful for updating UI, logging, or maintaining local state.
    /// Not required for decision-making (Partial is self-contained).
    async fn notify(&mut self, event: &Event);
}
