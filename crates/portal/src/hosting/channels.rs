use super::Client;
use super::RoomHandle;
use parlor::Room;
use pokerkit::ID;
use pokerkit::Pacing;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::oneshot;

/// Channels for room lifecycle coordination.
///
/// Every part leaves at once, so it hands them over as a tuple rather than
/// exposing five fields for a caller to take one at a time.
pub struct RoomChannels {
    handle: RoomHandle,
    client: Client,
    mirror: UnboundedSender<String>,
    skip: Arc<Notify>,
    start: oneshot::Receiver<()>,
}

impl RoomChannels {
    /// Pairs the two sides of a room: the handle the casino keeps, and the
    /// client, mirror and start signal the room runs on.
    pub fn pair(id: ID<Room>, pacing: Pacing) -> Self {
        let (outgoing, received) = unbounded_channel::<String>();
        let (incoming, accepted) = unbounded_channel::<String>();
        let (opening, opened) = oneshot::channel();
        let skip = Arc::new(Notify::new());
        Self {
            handle: RoomHandle::new(id, pacing, incoming, Arc::new(Mutex::new(received)), skip.clone(), opening),
            mirror: outgoing.clone(),
            client: Client::new(outgoing, Arc::new(Mutex::new(accepted))),
            skip,
            start: opened,
        }
    }

    pub fn into_parts(self) -> (RoomHandle, Client, UnboundedSender<String>, Arc<Notify>, oneshot::Receiver<()>) {
        let Self {
            handle,
            client,
            mirror,
            skip,
            start,
        } = self;
        (handle, client, mirror, skip, start)
    }
}
