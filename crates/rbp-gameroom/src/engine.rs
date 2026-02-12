use super::*;
use rbp_cards::*;
use rbp_core::*;
use rbp_gameplay::*;
use std::collections::HashSet;
use std::marker::PhantomData;
use tokio::sync::mpsc::*;

/// Phase: accepting players before game starts.
pub struct Seating;
/// Phase: hand in progress (dealing cards, collecting actions).
pub struct Dealing;
/// Phase: hand is terminal, executing showdown sequence.
pub struct Showdown;
/// Phase: game is over (a player busted).
pub struct Finished;

/// Functional core for a live poker game.
/// Maintains game state, enforces rules, and coordinates player communication.
/// Driven by Room (imperative shell) which handles persistence concerns.
///
/// Uses typestate pattern to encode valid phase transitions at compile time.
#[derive(Debug)]
pub struct Engine<Phase> {
    hand: u64,
    game: Game,
    history: Vec<Action>,
    channel: Channel<(usize, Event)>,
    players: Vec<UnboundedSender<Event>>,
    disconnected: HashSet<usize>,
    phase: PhantomData<Phase>,
}

impl Default for Engine<Seating> {
    fn default() -> Self {
        Self {
            hand: 0,
            game: Game::root(),
            channel: Channel::default(),
            players: Vec::new(),
            history: Vec::new(),
            disconnected: HashSet::new(),
            phase: PhantomData,
        }
    }
}

impl<T> Engine<T> {
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn players(&self) -> &Vec<UnboundedSender<Event>> {
        &self.players
    }
}

/// Seating phase: accepting players.
impl Engine<Seating> {
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
    /// Transition to Dealing phase. Broadcasts hand start and hole cards.
    pub fn start(self) -> Engine<Dealing> {
        let mut engine = Engine {
            hand: self.hand,
            game: self.game,
            history: self.history,
            channel: self.channel,
            players: self.players,
            disconnected: self.disconnected,
            phase: PhantomData,
        };
        engine.commence();
        engine
    }
}

/// Dealing phase: hand in progress.
impl Engine<Dealing> {
    pub fn turn(&self) -> Turn {
        self.game.turn()
    }
    pub fn hand(&self) -> u64 {
        self.hand
    }
    pub fn history(&self) -> &[Action] {
        &self.history
    }
    /// Deal next community cards (chance node).
    pub async fn deal(&mut self) {
        let reveal = self.game.reveal();
        log::debug!("[engine] dealing {}", reveal);
        self.apply(reveal);
        self.broadcast(Event::Board {
            hand: self.hand,
            street: self.game.street(),
            board: Hand::from(self.game.board()),
        });
    }
    /// Get decision from player at position (choice node).
    pub async fn ask(&mut self, pos: Position) -> Action {
        log::debug!("[engine] asking P{} for action", pos);
        self.unicast(
            pos,
            Event::Decision {
                hand: self.hand,
                recall: self.recall(pos),
            },
        );
        let action = self.next_action(pos).await;
        log::debug!("[engine] P{} chose {}", pos, action);
        self.apply(action);
        self.broadcast(Event::Action {
            hand: self.hand,
            seat: pos,
            action,
            pot: self.game.pot(),
        });
        action
    }
    /// Transition to Showdown phase when hand is terminal.
    pub fn into_showdown(self) -> Engine<Showdown> {
        Engine {
            hand: self.hand,
            game: self.game,
            history: self.history,
            channel: self.channel,
            players: self.players,
            disconnected: self.disconnected,
            phase: PhantomData,
        }
    }
}

/// Showdown phase: revealing cards and settling.
impl Engine<Showdown> {
    pub fn hand(&self) -> u64 {
        self.hand
    }
    pub fn history(&self) -> &[Action] {
        &self.history
    }
    /// Check if human player (position 0) has disconnected.
    pub fn human_disconnected(&self) -> bool {
        self.disconnected.contains(&0)
    }
    /// Returns true if this is a showdown (multiple players remain).
    pub fn is_showdown(&self) -> bool {
        self.game.is_showdown()
    }
    /// Executes showdown reveal sequence.
    pub async fn showdown(&mut self) {
        if !self.game.is_showdown() {
            return;
        }
        let forced = self.forced_reveals();
        let order = self.showdown_order();
        let mut showed = Vec::new();
        let deadline = tokio::time::Instant::now() + Self::showdown_timeout();
        loop {
            tokio::select! {
                biased;
                _ = tokio::time::sleep_until(deadline) => break,
                msg = self.channel.rx().recv() => {
                    if let Some((pos, _)) = msg {
                        if !showed.contains(&pos) && order.contains(&pos) {
                            let hole = self.game.seats()[pos].cards();
                            self.broadcast(Event::Reveal {
                                hand: self.hand,
                                seat: pos,
                                hole: Some(hole),
                            });
                            showed.push(pos);
                        }
                    }
                }
            }
        }
        forced
            .into_iter()
            .filter(|p| !showed.contains(p))
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|p| {
                self.broadcast(Event::Reveal {
                    hand: self.hand,
                    seat: p,
                    hole: Some(self.game.seats()[p].cards()),
                });
                showed.push(p);
            });
        order
            .into_iter()
            .filter(|p| !showed.contains(p))
            .for_each(|p| {
                self.broadcast(Event::Reveal {
                    hand: self.hand,
                    seat: p,
                    hole: None,
                })
            });
    }
    /// Broadcast hand end with winners.
    pub fn settle(&self) {
        let winners = self
            .game
            .settlements()
            .iter()
            .enumerate()
            .filter(|(_, s)| s.pnl().reward() > 0)
            .map(|(i, s)| (i, s.pnl().reward() as Chips))
            .collect();
        self.broadcast(Event::HandEnd {
            hand: self.hand,
            winners,
        });
    }
    /// Advance to next hand or finish.
    pub fn conclude(mut self) -> Result<Engine<Dealing>, Engine<Finished>> {
        match self.game.continuation() {
            Some(next) => {
                self.game = next;
                self.history.clear();
                self.hand += 1;
                let mut engine = Engine {
                    hand: self.hand,
                    game: self.game,
                    history: self.history,
                    channel: self.channel,
                    players: self.players,
                    disconnected: self.disconnected,
                    phase: PhantomData,
                };
                engine.commence();
                Ok(engine)
            }
            None => {
                log::info!("[engine] game over - a player is busted");
                Err(Engine {
                    hand: self.hand,
                    game: self.game,
                    history: self.history,
                    channel: self.channel,
                    players: self.players,
                    disconnected: self.disconnected,
                    phase: PhantomData,
                })
            }
        }
    }
    fn forced_reveals(&self) -> Vec<Position> {
        let settlements = self.game.settlements();
        self.game
            .seats()
            .iter()
            .enumerate()
            .filter(|(_, seat)| seat.state() != State::Folding)
            .filter(|(i, seat)| {
                seat.state() == State::Shoving
                    || settlements.get(*i).map_or(false, |s| s.pnl().reward() > 0)
            })
            .map(|(i, _)| i)
            .collect()
    }
    fn showdown_order(&self) -> Vec<Position> {
        let n = self.game.n();
        let first = self
            .recall(0)
            .aggressor()
            .unwrap_or_else(|| (self.game.dealer().position() + 1) % n);
        (0..n)
            .map(|i| (first + i) % n)
            .filter(|i| self.game.seats()[*i].state().is_active())
            .collect()
    }
    fn showdown_timeout() -> std::time::Duration {
        std::time::Duration::from_secs(SHOWDOWN_TIMEOUT)
    }
}

/// Finished phase: game is over.
impl Engine<Finished> {}

/// Private helpers shared across phases via macro.
macro_rules! impl_engine_internals {
    ($($phase:ty),*) => {
        $(
            impl Engine<$phase> {
                fn apply(&mut self, action: Action) {
                    self.history.push(action);
                    self.game = self.game.apply(action);
                }
                fn recall(&self, pos: Position) -> Partial {
                    Partial::from((
                        Turn::Choice(pos),
                        Observation::from((
                            Hand::from(self.game.seats().get(pos).expect("bounds").cards()),
                            Hand::from(self.game.board()),
                        )),
                        self.history
                            .iter()
                            .filter(|a| a.is_choice())
                            .cloned()
                            .collect(),
                    ))
                }
                fn unicast(&self, i: usize, event: Event) {
                    log::debug!("[engine] unicast to P{}: {}", i, event);
                    match self.players.get(i).map(|inbox| inbox.send(event)) {
                        Some(Ok(())) => log::debug!("[engine] unicast to P{} succeeded", i),
                        Some(Err(e)) => log::warn!("[engine] unicast to P{} failed: {:?}", i, e),
                        None => log::warn!("[engine] unicast to P{}: no such player", i),
                    }
                }
                fn broadcast(&self, event: Event) {
                    log::debug!("[engine] broadcast: {}", event);
                    self.players
                        .iter()
                        .enumerate()
                        .for_each(|(i, inbox)| match inbox.send(event.clone()) {
                            Ok(()) => {}
                            Err(e) => log::warn!("[engine] broadcast to P{} failed: {:?}", i, e),
                        });
                }
            }
        )*
    };
}
impl_engine_internals!(Dealing, Showdown);

/// Dealing-specific private helpers.
impl Engine<Dealing> {
    fn commence(&mut self) {
        self.broadcast(Event::HandStart {
            hand: self.hand,
            dealer: self.game.dealer().position(),
            stacks: self.game.seats().iter().map(|s| s.stack()).collect(),
        });
        self.game.seats().iter().enumerate().for_each(|(i, seat)| {
            self.unicast(
                i,
                Event::HoleCards {
                    hand: self.hand,
                    hole: seat.cards(),
                },
            )
        });
    }
    async fn next_action(&mut self, pos: Position) -> Action {
        loop {
            match self
                .poll_action(pos)
                .await
                .and_then(|a| self.validate(a).ok())
            {
                Some(a) => return a,
                None => continue,
            }
        }
    }
    async fn poll_action(&mut self, pos: Position) -> Option<Action> {
        tokio::time::timeout(Self::timeout(), self.channel.rx().recv())
            .await
            .inspect_err(|_| log::debug!("[engine] P{} timed out", pos))
            .unwrap_or_else(|_| {
                Some((
                    pos,
                    Event::Action {
                        hand: self.hand,
                        seat: pos,
                        action: self.game.passive(),
                        pot: self.game.pot(),
                    },
                ))
            })
            .filter(|(j, _)| *j == pos)
            .and_then(|(_, e)| match e {
                Event::Disconnect(p) => {
                    self.disconnected.insert(p);
                    Some(self.game.passive())
                }
                _ => e.action(),
            })
    }
    fn validate(&self, action: Action) -> Result<Action, Action> {
        self.game
            .is_allowed(&action)
            .then_some(action)
            .ok_or(action)
    }
    fn timeout() -> std::time::Duration {
        std::time::Duration::from_secs(10)
    }
}

/// Wrapper enum for Room to hold Engine in any phase.
pub enum EngineState {
    Seating(Engine<Seating>),
    Dealing(Engine<Dealing>),
    Showdown(Engine<Showdown>),
    Finished(Engine<Finished>),
}

impl Default for EngineState {
    fn default() -> Self {
        Self::Seating(Engine::default())
    }
}

impl EngineState {
    /// Unwrap as Seating phase, panics if wrong phase.
    pub fn as_seating(&mut self) -> &mut Engine<Seating> {
        match self {
            Self::Seating(e) => e,
            _ => panic!("expected Seating phase"),
        }
    }
    /// Take ownership and transition Seating -> Dealing.
    pub fn start(&mut self) {
        let state = std::mem::replace(self, EngineState::default());
        match state {
            Self::Seating(e) => *self = Self::Dealing(e.start()),
            _ => panic!("can only start from Seating phase"),
        }
    }
    /// Take ownership and transition Dealing -> Showdown.
    pub fn into_showdown(&mut self) {
        let state = std::mem::replace(self, EngineState::default());
        match state {
            Self::Dealing(e) => *self = Self::Showdown(e.into_showdown()),
            _ => panic!("can only enter showdown from Dealing phase"),
        }
    }
    /// Take ownership and transition Showdown -> Dealing or Finished.
    pub fn conclude(&mut self) {
        let state = std::mem::replace(self, EngineState::default());
        match state {
            Self::Showdown(e) => match e.conclude() {
                Ok(dealing) => *self = Self::Dealing(dealing),
                Err(finished) => *self = Self::Finished(finished),
            },
            _ => panic!("can only conclude from Showdown phase"),
        }
    }
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Finished(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn engine_default_is_seating() {
        let engine = Engine::<Seating>::default();
        assert_eq!(engine.game.pot(), Game::sblind() + Game::bblind());
    }
    #[test]
    fn engine_state_default_is_seating() {
        let state = EngineState::default();
        assert!(matches!(state, EngineState::Seating(_)));
    }
}
