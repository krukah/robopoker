use croupier::Action;
use croupier::Recall;
use croupier::ServerMessage;
use croupier::Witness;
use parlor::Player;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

/// Network player that communicates via tokio channels.
/// Designed to bridge WebSocket connections to the Room actor system.
///
/// `tx` ships JSON ServerMessages to the WebSocket client.
/// `rx` receives action strings from the WebSocket client.
///
/// Snapshot pushes from the engine flow through `tx` directly; this Player
/// only owns `tx` so it can deliver `Rejected` responses to invalid input.
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

    async fn decide(&mut self, recall: &Witness) -> Action {
        let legal = recall.head().legal();
        loop {
            match self.rx.lock().await.recv().await {
                None => {
                    self.alive.store(false, Ordering::SeqCst);
                    return recall.head().passive();
                }
                Some(s) => match Action::try_from(s.as_str()) {
                    Err(reason) => {
                        self.send(ServerMessage::rejected(reason, legal.clone()));
                    }
                    Ok(a) if !recall.head().is_allowed(&a) => {
                        self.send(ServerMessage::rejected("illegal action", legal.clone()));
                    }
                    Ok(a) => return a,
                },
            }
        }
    }
}
