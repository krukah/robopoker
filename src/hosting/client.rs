use crate::gameplay::Action;
use crate::gameplay::Recall;
use crate::gameroom::Event;
use crate::gameroom::Player;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

/// Network player that communicates via tokio channels.
/// Designed to bridge WebSocket connections to the Room actor system.
///
/// The tx channel sends JSON to the WebSocket client:
/// - Game state (Recall) when it's the player's turn
/// - Event notifications for all game actions
///
/// The rx channel receives JSON from the WebSocket client:
/// - Action decisions when prompted
pub struct Client {
    tx: UnboundedSender<String>,
    rx: Arc<Mutex<UnboundedReceiver<String>>>,
}

impl Client {
    pub fn new(tx: UnboundedSender<String>, rx: Arc<Mutex<UnboundedReceiver<String>>>) -> Self {
        Self { tx, rx }
    }
}

#[async_trait::async_trait]
impl Player for Client {
    async fn decide(&mut self, recall: &Recall) -> Action {
        let state = serde_json::json!({
            "type": "decision",
            "legal": recall.head().legal().iter().map(|a| a.to_string()).collect::<Vec<_>>(),
            "board": recall.board().iter().map(|c| c.to_string()).collect::<Vec<_>>(),
            "pot": recall.head().pot(),
        });
        self.tx
            .send(state.to_string())
            .inspect_err(|e| log::error!("failed to send decision state: {}", e))
            .ok();
        loop {
            match self
                .rx
                .lock()
                .await
                .recv()
                .await
                .and_then(|s| Action::try_from(s.as_str()).ok())
            {
                Some(action) if recall.head().is_allowed(&action) => return action,
                Some(_) => log::warn!("invalid action from client, retrying"),
                None => return recall.head().passive(),
            }
        }
    }
    async fn notify(&mut self, event: &Event) {
        let json = serde_json::json!({
            "type": "event",
            "event": format!("{:?}", event),
        });
        let _ = self.tx.send(json.to_string());
    }
}
