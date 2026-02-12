use super::*;
use tokio::sync::mpsc::*;

/// Wrapper that runs a Player in its own async task.
/// Handles message passing between Room and Player implementation.
///
/// - Room unicasts Decision when it's this player's turn
/// - Actor calls Player::decide and sends action back to Room
/// - Room broadcasts events for all game actions
/// - Actor forwards events to Player::notify
pub struct Actor {
    id: usize,
    player: Box<dyn Player>,
    getter: UnboundedReceiver<Event>,
    sender: UnboundedSender<(usize, Event)>,
}

impl Actor {
    pub fn spawn(
        id: usize,
        player: Box<dyn Player>,
        sender: UnboundedSender<(usize, Event)>,
    ) -> UnboundedSender<Event> {
        let (tx, rx) = unbounded_channel();
        let actor = Self {
            id,
            player,
            sender,
            getter: rx,
        };
        tokio::spawn(actor.run());
        tx
    }
    async fn run(mut self) {
        loop {
            match self.getter.recv().await {
                Some(ref event @ Event::Decision { ref recall, .. }) => {
                    log::debug!("[actor P{}] received Decision", self.id);
                    self.player.notify(event).await;
                    self.act(recall).await;
                    if !self.player.alive() {
                        log::info!("[actor P{}] player disconnected", self.id);
                        let _ = self.sender.send((self.id, Event::Disconnect(self.id)));
                        break;
                    }
                }
                Some(ref event) => {
                    log::trace!("[actor P{}] received {}", self.id, event);
                    self.player.notify(event).await;
                }
                None => break,
            }
        }
    }
    async fn act(&mut self, recall: &rbp_gameplay::Partial) {
        log::debug!("[actor P{}] calling decide", self.id);
        let action = self.player.decide(recall).await;
        log::debug!("[actor P{}] decided {}", self.id, action);
        let event = Event::Action {
            hand: 0,
            seat: self.id,
            action,
            pot: 0,
        };
        let _ = self.sender.send((self.id, event));
    }
}
