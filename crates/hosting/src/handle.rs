use super::Client;
use rbp_core::ID;
use rbp_gameroom::Room;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::oneshot;

/// Handle to communicate with a running room.
/// Stores channel endpoints for bridging WebSocket to Client player.
pub struct RoomHandle {
    pub id: ID<Room>,
    pub tx: UnboundedSender<String>,
    pub rx: Arc<Mutex<UnboundedReceiver<String>>>,
    pub start: Option<oneshot::Sender<()>>,
}

/// Channels for room lifecycle coordination.
pub struct RoomChannels {
    pub handle: RoomHandle,
    pub client: Client,
    pub start: oneshot::Receiver<()>,
    pub done_tx: oneshot::Sender<()>,
    pub done_rx: oneshot::Receiver<()>,
}

impl RoomHandle {
    /// Creates paired channels for room communication.
    /// Returns channels for Casino (handle, done_rx), Room (client, start, done_tx).
    pub fn pair(id: ID<Room>) -> RoomChannels {
        let (tx_outgoing, rx_outgoing) = unbounded_channel::<String>();
        let (tx_incoming, rx_incoming) = unbounded_channel::<String>();
        let (start_tx, start_rx) = oneshot::channel();
        let (done_tx, done_rx) = oneshot::channel();
        let client = Client::new(tx_outgoing, Arc::new(Mutex::new(rx_incoming)));
        let handle = RoomHandle {
            id,
            tx: tx_incoming,
            rx: Arc::new(Mutex::new(rx_outgoing)),
            start: Some(start_tx),
        };
        RoomChannels {
            handle,
            client,
            start: start_rx,
            done_tx,
            done_rx,
        }
    }
}
