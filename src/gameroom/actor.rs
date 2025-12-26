use super::*;
use tokio::sync::mpsc::*;

/// Wrapper that runs a Player in its own async task.
/// Handles message passing between Room and Player implementation.
///
/// - Room unicasts YourTurn(Recall) when it's this player's turn
/// - Actor calls Player::decide and sends action back to Room
/// - Room broadcasts Play events for all game actions
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
    async fn run(mut self) -> ! {
        loop {
            match self.getter.recv().await {
                Some(Event::YourTurn(ref recall)) => self.act(recall).await,
                Some(ref event) => self.player.notify(event).await,
                None => continue,
            }
        }
    }
    async fn act(&mut self, recall: &crate::gameplay::Recall) {
        let action = self.player.decide(recall).await;
        let _ = self.sender.send((self.id, Event::Play(action)));
    }
}
