use rbp_gameplay::Action;
use rbp_gameplay::Partial;
use rbp_gameplay::Recall;
use rbp_gameroom::Event;
use rbp_gameroom::Player;
use rbp_gameroom::Protocol;
use rbp_gameroom::ServerMessage;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

/// Network player that communicates via tokio channels.
/// Designed to bridge WebSocket connections to the Room actor system.
///
/// The tx channel sends JSON (ServerMessage) to the WebSocket client.
/// The rx channel receives action strings from the WebSocket client.
pub struct Client {
    tx: UnboundedSender<String>,
    rx: Arc<Mutex<UnboundedReceiver<String>>>,
    alive: Arc<AtomicBool>,
}

impl Client {
    pub fn new(tx: UnboundedSender<String>, rx: Arc<Mutex<UnboundedReceiver<String>>>) -> Self {
        Self {
            tx,
            rx,
            alive: Arc::new(AtomicBool::new(true)),
        }
    }
    fn send(&self, msg: ServerMessage) {
        let _ = self.tx.send(msg.to_json());
    }
}

#[async_trait::async_trait]
impl Player for Client {
    fn alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
    async fn decide(&mut self, recall: &Partial) -> Action {
        loop {
            match self.rx.lock().await.recv().await {
                None => {
                    self.alive.store(false, Ordering::SeqCst);
                    return recall.head().passive();
                }
                Some(s) => match Action::try_from(s.as_str())
                    .ok()
                    .filter(|a| recall.head().is_allowed(a))
                {
                    Some(a) => return a,
                    None => continue,
                },
            }
        }
    }
    async fn notify(&mut self, event: &Event) {
        if let Some(msg) = Protocol::encode(event) {
            self.send(msg);
        }
    }
}
