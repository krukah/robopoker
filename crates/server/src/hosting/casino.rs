use super::*;
use rbp_auth::Lurker;
use rbp_core::ID;
use rbp_core::Variant;
use rbp_gameplay::ServerMessage;
use rbp_gameroom::VariantExt;
use rbp_gameroom::*;
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
///
/// `blueprint` is optional: when `None`, only opponents that don't need a
/// blueprint (e.g. [`Variant::Fish`]) can be spawned. This lets a backend run
/// without the heavy in-memory blueprint hydration step.
pub struct Casino {
    db: Arc<Client>,
    blueprint: Option<&'static rbp_nlhe::Flagship>,
    rooms: RwLock<HashMap<ID<Room>, RoomHandle>>,
}

impl Casino {
    pub fn new(db: Arc<Client>) -> Self {
        Self {
            db,
            blueprint: None,
            rooms: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_blueprint(mut self, blueprint: Option<&'static rbp_nlhe::Flagship>) -> Self {
        self.blueprint = blueprint;
        self
    }
}

impl Casino {
    /// Opens a new room with HTTP client vs the chosen [`Variant`].
    /// Spawns the room task (waits for start signal) and returns the room ID.
    pub async fn start(self: &Arc<Self>, variant: Variant) -> anyhow::Result<ID<Room>> {
        if variant.requires_blueprint() && self.blueprint.is_none() {
            anyhow::bail!("variant {} needs a blueprint but this Casino was started without one", variant.label());
        }
        let id = ID::default();
        let channels = RoomHandle::pair(id);
        let mut engine = Engine::<Seating>::default();
        engine.set_skip(channels.skip.clone());
        let mut room = Room::new(id, 2, self.db.clone());
        self.db.create_room(&room).await?;
        self.rooms.write().await.insert(id, channels.handle);
        room.sit(&mut engine, channels.client, Lurker::default(), Some(channels.mirror));
        let player = variant.into_player(self.blueprint);
        room.sit(&mut engine, player, variant.user(), None);
        let handle = tokio::spawn(room.run(engine, channels.start));
        let casino = self.clone();
        tokio::spawn(async move {
            let _ = handle.await;
            let _ = casino.close(id).await;
            tracing::info!(room = %id, "room cleaned up");
        });
        tracing::debug!(room = %id, variant = variant.label(), "created room");
        Ok(id)
    }
    /// Closes a room and removes it from the casino.
    pub async fn close(&self, id: ID<Room>) -> anyhow::Result<()> {
        self.rooms
            .write()
            .await
            .remove(&id)
            .map(|h| {
                if let Some(b) = h.bridge {
                    b.abort();
                }
            })
            .ok_or_else(|| anyhow::anyhow!("room not found"))
    }
    /// Gets channel endpoints and start signal for WebSocket bridging.
    pub async fn channels(
        &self,
        id: ID<Room>,
    ) -> anyhow::Result<(Tx, Rx, Arc<tokio::sync::Notify>, Option<tokio::sync::oneshot::Sender<()>>)> {
        self.rooms
            .write()
            .await
            .get_mut(&id)
            .map(|h| (h.tx.clone(), h.rx.clone(), h.skip.clone(), h.start.take()))
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
        let (tx, rx, skip, start) = self.channels(id).await?;
        session
            .text(ServerMessage::welcome(&id.to_string(), 0).to_json())
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        start.map(|s| s.send(()));
        tracing::debug!(bridge = %id, "connected");
        let task = actix_web::rt::spawn(async move {
            'sesh: loop {
                tokio::select! {
                    biased;
                    msg = async { rx.lock().await.recv().await } => match msg {
                        Some(json) => if session.text(json).await.is_err() {
                            tracing::warn!(bridge = %id, "send to client failed");
                            break 'sesh;
                        },
                        None => break 'sesh,
                    },
                    msg = streams.next() => match msg {
                        Some(Ok(actix_ws::Message::Text(text))) => {
                            if &*text == "skip" {
                                skip.notify_one();
                            } else if tx.send(text.to_string()).is_err() {
                                break 'sesh;
                            }
                        },
                        Some(Ok(actix_ws::Message::Close(reason))) => {
                            tracing::debug!(bridge = %id, ?reason, "client closed");
                            break 'sesh;
                        },
                        Some(Err(e)) => {
                            tracing::warn!(bridge = %id, error = ?e, "read error");
                            break 'sesh;
                        },
                        None => break 'sesh,
                        _ => continue 'sesh,
                    },
                }
            }
            tracing::debug!(bridge = %id, "disconnected");
        });
        self.rooms
            .write()
            .await
            .get_mut(&id)
            .map(|h| h.bridge = Some(task.abort_handle()));
        Ok(())
    }
}
