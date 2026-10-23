use parlor::Room;
use pokerkit::ID;
use pokerkit::Pacing;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

/// Handle to communicate with a running room.
/// Stores channel endpoints for bridging WebSocket to Client player.
pub struct RoomHandle {
    id: ID<Room>,
    pacing: Pacing,
    tx: UnboundedSender<String>,
    rx: Arc<Mutex<UnboundedReceiver<String>>>,
    skip: Arc<Notify>,
    start: Option<oneshot::Sender<()>>,
    bridge: Option<tokio::task::AbortHandle>,
}

impl RoomHandle {
    pub fn new(
        id: ID<Room>,
        pacing: Pacing,
        tx: UnboundedSender<String>,
        rx: Arc<Mutex<UnboundedReceiver<String>>>,
        skip: Arc<Notify>,
        start: oneshot::Sender<()>,
    ) -> Self {
        Self {
            id,
            pacing,
            tx,
            rx,
            skip,
            start: Some(start),
            bridge: None,
        }
    }

    pub fn id(&self) -> ID<Room> {
        self.id
    }

    /// How the room it fronts is paced. Read off the engine at creation, so
    /// a room paced at any tempo tells the truth to whoever connects.
    pub fn pacing(&self) -> Pacing {
        self.pacing
    }

    pub fn tx(&self) -> UnboundedSender<String> {
        self.tx.clone()
    }

    pub fn rx(&self) -> Arc<Mutex<UnboundedReceiver<String>>> {
        self.rx.clone()
    }

    pub fn skip(&self) -> Arc<Notify> {
        self.skip.clone()
    }

    /// The start signal, which fires once. Taking it is the point: the
    /// second connection to a room must not start it again.
    pub fn start(&mut self) -> Option<oneshot::Sender<()>> {
        self.start.take()
    }

    fn bridge(&self) -> Option<&tokio::task::AbortHandle> {
        self.bridge.as_ref()
    }

    fn bridge_mut(&mut self) -> &mut Option<tokio::task::AbortHandle> {
        &mut self.bridge
    }

    /// A reconnect replaces the bridge; the stale task would otherwise keep draining the shared
    /// receiver and escape `abort`.
    pub fn watch(&mut self, bridge: tokio::task::AbortHandle) {
        if let Some(stale) = self.bridge_mut().replace(bridge) {
            stale.abort();
        }
    }

    pub fn abort(&self) {
        self.bridge().map(tokio::task::AbortHandle::abort);
    }
}
