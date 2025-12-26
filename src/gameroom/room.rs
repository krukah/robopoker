use super::*;
use crate::cards::*;
use crate::gameplay::*;
use tokio::sync::mpsc::*;

/// Central coordinator for a live poker game.
/// Maintains single source of truth (Game state + action history),
/// orchestrates turn-taking, and broadcasts events to all players.
///
/// Key responsibilities:
/// - Enforce game rules and turn order
/// - Maintain complete action history
/// - Broadcast Event for all game actions
/// - Handle player timeouts
/// - Coordinate async player decisions via input/output channels
///
/// Room runs in a loop processing game states:
/// - Terminal: Settle hand, broadcast settlements, start new hand
/// - Chance: Deal cards, broadcast action
/// - Choice: Wait for decision from acting player with timeout
///
/// Clients derive turn state from action stream, eliminating YourTurn events.
#[derive(Debug)]
pub struct Room {
    game: Game,
    history: Vec<Action>,
    channel: Channel<(usize, Event)>,
    players: Vec<UnboundedSender<Event>>,
}

impl Default for Room {
    fn default() -> Self {
        Self {
            game: Game::root(),
            channel: Channel::default(),
            players: Vec::new(),
            history: Vec::new(),
        }
    }
}

impl Room {
    pub async fn run(mut self) -> ! {
        loop {
            if self.history.is_empty() {
                self.game()
                    .seats()
                    .iter()
                    .enumerate()
                    .for_each(|(i, seat)| self.unicast(i, Event::ShowHand(i, seat.cards())));
            }
            match self.game().turn() {
                Turn::Chance => self.next_card().await,
                Turn::Terminal => self.next_hand().await,
                Turn::Choice(i) => self.next_turn(i).await,
            }
        }
    }

    pub fn sit<P>(&mut self, player: P)
    where
        P: Player + 'static,
    {
        self.players.push(Actor::spawn(
            self.players.len(),
            Box::new(player),
            self.channel.tx().clone(),
        ));
    }
}

impl Room {
    async fn next_turn(&mut self, pos: usize) {
        let action = self.ask(pos).await;
        self.apply(action);
        self.broadcast(Event::Play(action));
    }

    async fn next_card(&mut self) {
        let reveal = self.game().reveal();
        self.apply(reveal);
        self.broadcast(Event::Play(reveal));
    }

    async fn next_hand(&mut self) {
        self.game = self
            .game
            .next()
            .ok_or_else(|| anyhow::anyhow!("no next hand available, restarting"))
            .inspect_err(|e| log::warn!("{}", e))
            .unwrap_or_else(|_| Game::root());
        self.history.clear();
    }
}

impl Room {
    async fn ask(&mut self, i: usize) -> Action {
        self.unicast(i, Event::YourTurn(self.recall(i)));
        loop {
            match tokio::time::timeout(Self::timeout(), self.channel.rx().recv())
                .await
                .unwrap_or_else(|_| Some((i, Event::Play(self.game().passive()))))
                .filter(|(j, _)| *j == i)
                .map(|(_, e)| e)
            {
                Some(Event::Play(action)) if self.game().is_allowed(&action) => return action,
                Some(Event::Play(_)) => return self.game().passive(),
                _ => continue,
            }
        }
    }

    fn apply(&mut self, action: Action) {
        self.history.push(action);
        self.game = self.game().apply(action);
    }

    fn recall(&self, pos: usize) -> Recall {
        Recall::from((
            Turn::Choice(pos),
            Observation::from((
                Hand::from(self.game().seats().get(pos).expect("bounds").cards()),
                Hand::from(self.game().board()),
            )),
            self.history.clone(),
        ))
    }
}

impl Room {
    fn unicast(&self, i: usize, event: Event) {
        self.players
            .get(i)
            .map(|inbox| inbox.send(event))
            .and_then(|res| res.err())
            .inspect(|e| log::warn!("failed unicast to P{}: {:?}", i, e));
    }
    fn broadcast(&self, event: Event) {
        self.players
            .iter()
            .map(|inbox| inbox.send(event.clone()))
            .collect::<Vec<Result<_, _>>>()
            .into_iter()
            .enumerate()
            .filter_map(|(i, res)| res.err().map(|e| (i, e)))
            .for_each(|(i, e)| log::warn!("failed broadcast to P{}: {:?}", i, e));
    }
}

impl Room {
    fn timeout() -> std::time::Duration {
        std::time::Duration::from_secs(60)
    }
    fn game(&self) -> &Game {
        &self.game
    }
}
