use super::*;
use rbp_auth::Lurker;
use rbp_core::ID;
use rbp_gameroom::ServerMessage;
use rbp_gameroom::*;
use rbp_players::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio_postgres::Client;

type Tx = UnboundedSender<String>;
type Rx = Arc<Mutex<UnboundedReceiver<String>>>;

/// Manages active game rooms and their lifecycles.
pub struct Casino {
    db: Arc<Client>,
    rooms: RwLock<HashMap<ID<Room>, RoomHandle>>,
}

impl Casino {
    pub fn new(db: Arc<Client>) -> Self {
        Self {
            db,
            rooms: RwLock::new(HashMap::new()),
        }
    }
}

impl Casino {
    /// Opens a new room with HTTP client vs Fish CPU.
    /// Spawns the room task (waits for start signal) and returns the room ID.
    pub async fn start(self: &Arc<Self>) -> anyhow::Result<ID<Room>> {
        let id = ID::default();
        let channels = RoomHandle::pair(id);
        let mut room = Room::new(id, 2, self.db.clone());
        self.db.create_room(&room).await?;
        self.rooms.write().await.insert(id, channels.handle);
        room.sit(channels.client, Lurker::default());
        room.sit(Fish, Lurker::default());
        tokio::spawn(room.run(channels.start, channels.done_tx));
        let casino = self.clone();
        tokio::spawn(async move {
            let _ = channels.done_rx.await;
            let _ = casino.close(id).await;
            log::info!("[casino] room {} cleaned up", id);
        });
        log::debug!("[casino] created room {}", id);
        Ok(id)
    }
    /// Closes a room and removes it from the casino.
    pub async fn close(&self, id: ID<Room>) -> anyhow::Result<()> {
        self.rooms
            .write()
            .await
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| anyhow::anyhow!("room not found"))
    }
    /// Gets channel endpoints and start signal for WebSocket bridging.
    pub async fn channels(
        &self,
        id: ID<Room>,
    ) -> anyhow::Result<(Tx, Rx, Option<tokio::sync::oneshot::Sender<()>>)> {
        self.rooms
            .write()
            .await
            .get_mut(&id)
            .map(|h| (h.tx.clone(), h.rx.clone(), h.start.take()))
            .ok_or_else(|| anyhow::anyhow!("room not found"))
    }
    /// Spawns WebSocket bridge between client and room channels.
    /// Sends start signal to room when first client connects.
    pub async fn bridge(
        &self,
        id: ID<Room>,
        mut session: actix_ws::Session,
        mut streams: actix_ws::MessageStream,
    ) -> anyhow::Result<()> {
        use futures::StreamExt;
        let (tx, rx, start) = self.channels(id).await?;
        session
            .text(ServerMessage::connected(&id.to_string(), 0).to_json())
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        start.map(|s| s.send(()));
        log::debug!("[bridge {}] connected", id);
        actix_web::rt::spawn(async move {
            'sesh: loop {
                tokio::select! {
                    biased;
                    msg = async { rx.lock().await.recv().await } => match msg {
                        Some(json) => if session.text(json).await.is_err() { break 'sesh },
                        None => break 'sesh,
                    },
                    msg = streams.next() => match msg {
                        Some(Ok(actix_ws::Message::Text(text))) => if tx.send(text.to_string()).is_err() { break 'sesh },
                        Some(Ok(actix_ws::Message::Close(_))) => break 'sesh,
                        Some(Err(_)) => break 'sesh,
                        None => break 'sesh,
                        _ => continue 'sesh,
                    },
                }
            }
            log::debug!("[bridge {}] disconnected", id);
        });
        Ok(())
    }
}
