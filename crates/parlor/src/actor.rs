use super::*;
use tokio::sync::mpsc::*;

enum Outcome {
    Decided(cowboys::Action),
    Restart(cowboys::Witness),
}

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
        let handle = tokio::spawn(actor.run());
        tokio::spawn(async move {
            handle
                .await
                .inspect_err(|e| tracing::error!(seat = id, error = %e, "actor task panicked"))
                .ok();
        });
        tx
    }

    #[tracing::instrument(skip_all, fields(seat = self.id))]
    async fn run(mut self) {
        loop {
            match self.getter.recv().await {
                Some(ref event @ Event::Decision(ref recall)) => {
                    tracing::debug!("received Decision");
                    self.player.notify(event).await;
                    self.act(recall).await;
                    if !self.player.alive() {
                        tracing::info!("player disconnected");
                        let _ = self.sender.send((self.id, Event::Disconnect(self.id)));
                        break;
                    }
                }
                Some(ref event) => {
                    tracing::trace!(%event, "received event");
                    self.player.notify(event).await;
                }
                None => break,
            }
        }
    }

    async fn act(&mut self, initial: &cowboys::Witness) {
        tracing::debug!("calling decide");
        let pace = self.player.pace();
        let mut recall = initial.clone();
        let action = loop {
            let outcome = {
                let decide = self.player.decide(&recall);
                tokio::pin!(decide);
                loop {
                    tokio::select! {
                        biased;
                        action = &mut decide => break Outcome::Decided(action),
                        event = self.getter.recv() => {
                            if let Some(Event::Decision(next)) = event {
                                break Outcome::Restart(next);
                            }
                        }
                    }
                }
            };
            match outcome {
                Outcome::Decided(action) => break action,
                Outcome::Restart(next) => {
                    tracing::debug!("restarting decide (stale recall replaced)");
                    recall = next;
                }
            }
        };
        tracing::debug!(%action, "decided");
        tokio::time::sleep(pace).await;
        let _ = self.sender.send((self.id, Event::Action(action)));
    }
}
