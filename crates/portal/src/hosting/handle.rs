use super::Client;
use parlor::Room;
use pokerkit::ID;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;
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
    pub skip: Arc<Notify>,
    pub start: Option<oneshot::Sender<()>>,
    pub bridge: Option<tokio::task::AbortHandle>,
}

/// Channels for room lifecycle coordination.
pub struct RoomChannels {
    pub handle: RoomHandle,
    pub client: Client,
    pub mirror: UnboundedSender<String>,
    pub skip: Arc<Notify>,
    pub start: oneshot::Receiver<()>,
}

impl RoomHandle {
    /// Creates paired channels for room communication.
    /// Returns channels for Casino (handle), Room (client, start).
    pub fn pair(id: ID<Room>) -> RoomChannels {
        let (tx_outgoing, rx_outgoing) = unbounded_channel::<String>();
        let (tx_incoming, rx_incoming) = unbounded_channel::<String>();
        let (start_tx, start_rx) = oneshot::channel();
        let skip = Arc::new(Notify::new());
        let mirror = tx_outgoing.clone();
        let client = Client::new(tx_outgoing, Arc::new(Mutex::new(rx_incoming)));
        let handle = RoomHandle {
            id,
            tx: tx_incoming,
            rx: Arc::new(Mutex::new(rx_outgoing)),
            skip: skip.clone(),
            start: Some(start_tx),
            bridge: None,
        };
        RoomChannels {
            handle,
            client,
            mirror,
            skip,
            start: start_rx,
        }
    }
}
