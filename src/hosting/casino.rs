use super::*;
use crate::gameroom::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

type Tx = UnboundedSender<String>;
type Rx = Arc<Mutex<UnboundedReceiver<String>>>;

/// Manages active game rooms and their lifecycles.
pub struct Casino {
    rooms: RwLock<HashMap<RoomId, RoomHandle>>,
    count: AtomicU64,
}

impl Default for Casino {
    fn default() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            count: AtomicU64::new(1),
        }
    }
}

impl Casino {
    /// Opens a new room with HTTP client vs Fish CPU.
    /// Spawns the room task and returns the room ID.
    pub async fn start(&self) -> anyhow::Result<RoomId> {
        let id = self.count.fetch_add(1, Ordering::Relaxed);
        let (handle, client) = RoomHandle::pair(id);
        let mut room = Room::default();
        self.rooms.write().await.insert(id, handle);
        room.sit(client);
        room.sit(Fish);
        tokio::spawn(room.run());
        Ok(id).inspect(|_| log::info!("opened room {}", id))
    }

    /// Closes a room and removes it from the casino.
    pub async fn close(&self, id: RoomId) -> anyhow::Result<()> {
        self.rooms
            .write()
            .await
            .remove(&id)
            .map(|_| log::info!("closed room {}", id))
            .ok_or_else(|| anyhow::anyhow!("room not found"))
    }

    /// Gets channel endpoints for WebSocket bridging.
    pub async fn channels(&self, id: RoomId) -> anyhow::Result<(Tx, Rx)> {
        self.rooms
            .read()
            .await
            .get(&id)
            .map(|h| (h.tx.clone(), h.rx.clone()))
            .ok_or_else(|| anyhow::anyhow!("room not found"))
    }

    /// Spawns WebSocket bridge between client and room channels.
    pub async fn bridge(
        &self,
        id: RoomId,
        mut session: actix_ws::Session,
        mut stream: actix_ws::MessageStream,
    ) -> anyhow::Result<()> {
        use futures::StreamExt;
        let (tx, rx) = self
            .channels(id)
            .await
            .inspect(|_| log::info!("client connected to room {}", id))?;
        actix_web::rt::spawn(async move {
            'sesh: loop {
                tokio::select! {
                    biased;
                    msg = async { rx.lock().await.recv().await } => match msg {
                        Some(json) => if session.text(json).await.is_err() { break 'sesh },
                        None => break 'sesh,
                    },
                    msg = stream.next() => match msg {
                        Some(Ok(actix_ws::Message::Text(text))) => if tx.send(text.to_string()).is_err() { break 'sesh },
                        Some(Ok(actix_ws::Message::Close(_))) => break 'sesh,
                        Some(Err(_)) => break 'sesh,
                        None => break 'sesh,
                        _ => continue 'sesh,
                    },
                }
            }
        });
        Ok(())
    }
}
