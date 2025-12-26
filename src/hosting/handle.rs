use super::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

pub type RoomId = u64;

/// Handle to communicate with a running room.
/// Stores channel endpoints for bridging WebSocket to Client player.
pub struct RoomHandle {
    pub id: RoomId,
    pub tx: UnboundedSender<String>,
    pub rx: Arc<Mutex<UnboundedReceiver<String>>>,
}

impl RoomHandle {
    /// Creates paired channels for room communication.
    /// Returns the handle (for State) and client player (for Room).
    pub fn pair(id: RoomId) -> (Self, Client) {
        let (tx_outgoing, rx_outgoing) = unbounded_channel::<String>();
        let (tx_incoming, rx_incoming) = unbounded_channel::<String>();
        let client = Client::new(tx_outgoing, Arc::new(Mutex::new(rx_incoming)));
        let handle = Self {
            id,
            tx: tx_incoming,
            rx: Arc::new(Mutex::new(rx_outgoing)),
        };
        (handle, client)
    }
}
