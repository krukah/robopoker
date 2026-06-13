use super::*;
use rbp_core::*;
use rbp_gameplay::*;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::mpsc::*;

type Inbox = (usize, Event);

/// Phase: accepting players before game starts.
pub struct Seating;
/// Phase: hand in progress (dealing cards, collecting actions).
pub struct Dealing;
/// Phase: hand is terminal, executing showdown sequence.
pub struct Showdown;
/// Phase: game is over (a player busted).
pub struct Finished;

/// Shared state across all engine phases.
struct EngineCore {
    live: LiveGame,
    timing: TimerConfig,
    skip: Arc<Notify>,
    tx: UnboundedSender<Inbox>,
    rx: UnboundedReceiver<Inbox>,
    players: Vec<UnboundedSender<Event>>,
    wires: Vec<Option<UnboundedSender<String>>>,
    history: Vec<CompletedHand>,
    disconnected: HashSet<usize>,
    showoffs: HashSet<usize>,
}

impl EngineCore {
    async fn interruptible(&self, duration: std::time::Duration) {
        tokio::select! {
            biased;
            () = self.skip.notified() => {},
            () = tokio::time::sleep(duration) => {},
        }
    }

    fn apply_action(&mut self, action: Action) {
        self.live.act(action);
    }

    fn recall(&self, pos: Position) -> Witness {
        let cards = self
            .live
            .game()
            .seats()
            .get(pos)
            .unwrap()
            .cards()
            .into_iter()
            .chain(self.live.dealt())
            .collect::<Vec<_>>();
        self.live.actions().iter().filter(|a| a.is_choice()).copied().fold(
            Witness::initial_with(
                Turn::Choice(pos),
                Arrangement::from(cards),
                self.live.root().buyins(),
                self.live.root().dealer().position(),
            ),
            |r, a| r.push(a),
        )
    }

    fn snapshot(&self, pos: Position) -> Snapshot {
        Snapshot {
            hand: self.live.epoch(),
            phase: self.live.phase(),
            witness: self.recall(pos),
            reveals: (0..self.live.holes().len())
                .map(|i| if i == pos { None } else { self.live.shown(i) })
                .collect(),
            settlements: self.live.settlements().to_vec(),
            history: self.history.clone(),
        }
    }

    fn push_snapshots(&self) {
        for pos in 0..self.wires.len() {
            self.send_wire(pos, ServerMessage::Snapshot(self.snapshot(pos)));
        }
    }

    fn push_session_end(&self, stacks: [Chips; N], reason: Reason) {
        for pos in 0..self.wires.len() {
            self.send_wire(pos, ServerMessage::session_end(stacks, reason));
        }
    }

    fn send_wire(&self, pos: Position, msg: ServerMessage) {
        if let Some(tx) = self.wires.get(pos).and_then(|w| w.as_ref())
            && let Err(e) = tx.send(msg.to_json())
        {
            tracing::warn!(seat = pos, error = ?e, "wire send failed");
        }
    }

    fn record_hand_closing(&mut self) {
        self.history.push(CompletedHand {
            epoch: self.live.epoch(),
            settlements: self.live.settlements().to_vec(),
        });
    }

    async fn commence(&mut self) {
        self.interruptible(self.timing.deal_hole).await;
        for i in 0..self.players.len() {
            let hole = self.live.game().seats()[i].cards();
            self.live.deal_hole(i, hole);
        }
        self.push_snapshots();
    }

    fn unicast(&self, i: usize, event: Event) {
        tracing::trace!(seat = i, %event, "unicast");
        match self.players.get(i).map(|inbox| inbox.send(event)) {
            Some(Ok(())) => {}
            Some(Err(e)) => tracing::warn!(seat = i, error = ?e, "unicast failed"),
            None => tracing::warn!(seat = i, "unicast: no such player"),
        }
    }
}

impl Recall for EngineCore {
    fn root(&self) -> Game {
        *self.live.root()
    }

    fn actions(&self) -> &[Action] {
        self.live.actions()
    }
}

/// Functional core for a live poker game.
/// Maintains game state, enforces rules, and coordinates player communication.
/// Driven by Room (imperative shell) which handles persistence concerns.
///
/// Uses typestate pattern to encode valid phase transitions at compile time.
pub struct Engine<Phase> {
    core: EngineCore,
    phase: PhantomData<Phase>,
}

impl Default for Engine<Seating> {
    fn default() -> Self {
        Self {
            core: {
                let (tx, rx) = unbounded_channel();
                EngineCore {
                    live: LiveGame::default(),
                    timing: TimerConfig::default(),
                    skip: Arc::new(Notify::new()),
                    tx,
                    rx,
                    players: Vec::new(),
                    wires: Vec::new(),
                    history: Vec::new(),
                    disconnected: HashSet::new(),
                    showoffs: HashSet::new(),
                }
            },
            phase: PhantomData,
        }
    }
}

impl<T> Recall for Engine<T> {
    fn root(&self) -> Game {
        *self.core.live.root()
    }

    fn actions(&self) -> &[Action] {
        self.core.live.actions()
    }
}

/// Phase-agnostic accessors available in any phase.
impl<T> Engine<T> {
    pub fn game(&self) -> Game {
        self.core.live.game()
    }

    pub fn hand(&self) -> u64 {
        self.core.live.epoch()
    }

    pub fn timing(&self) -> &TimerConfig {
        &self.core.timing
    }

    pub fn skip(&self) -> &Arc<Notify> {
        &self.core.skip
    }

    pub fn is_disconnected(&self, pos: usize) -> bool {
        self.core.disconnected.contains(&pos)
    }

    pub fn final_stacks(&self) -> [Chips; N] {
        let game = self.core.live.game();
        let settlements = game.settlements();
        let seats = game.seats();
        std::array::from_fn(|i| seats[i].stack() + settlements[i].pnl().reward())
    }

    pub fn end_session(&self, stacks: [Chips; N], reason: Reason) {
        self.core.push_session_end(stacks, reason);
    }
}

/// Seating phase: accepting players.
impl Engine<Seating> {
    pub fn set_skip(&mut self, skip: Arc<Notify>) {
        self.core.skip = skip;
    }

    pub fn sit<P>(&mut self, player: P, wire: Option<UnboundedSender<String>>)
    where
        P: Player + 'static,
    {
        let pos = self.core.players.len();
        if player.shows() {
            self.core.showoffs.insert(pos);
        }
        self.core
            .players
            .push(Actor::spawn(pos, Box::new(player), self.core.tx.clone()));
        self.core.wires.push(wire);
    }

    /// Transition to Dealing phase. Broadcasts hand start and hole cards.
    pub async fn start(mut self) -> Engine<Dealing> {
        self.core.commence().await;
        Engine {
            core: self.core,
            phase: PhantomData,
        }
    }
}

/// Dealing phase: hand in progress.
impl Engine<Dealing> {
    pub fn turn(&self) -> Turn {
        self.core.live.game().turn()
    }

    /// Returns the most recent Draw action, if the last action was a draw.
    pub fn last_draw(&self) -> Option<Action> {
        self.core.live.actions().last().filter(|a| a.is_chance()).copied()
    }

    /// Deal next community cards (chance node).
    pub async fn deal(&mut self) {
        let hand = self.core.live.game().reveal().hand().unwrap();
        tracing::debug!(?hand, "dealing");
        self.core.live.deal(hand);
        self.core.push_snapshots();
        self.core.interruptible(self.core.timing.deal_board).await;
    }

    /// Get decision from player at position (choice node).
    /// Returns the action taken and how it was prompted.
    pub async fn ask(&mut self, pos: Position) -> (Action, Prompt) {
        tracing::debug!(seat = pos, "asking for action");
        // Tell the player actor it's their turn so internal Players can decide.
        self.core.unicast(pos, Event::Decision(self.core.recall(pos)));
        // Push fresh snapshot so the wire learns who to_act is + sees legal moves.
        self.core.push_snapshots();
        let (action, prompt) = self.next_action(pos).await;
        tracing::debug!(
            seat = pos,
            %action,
            timeout = prompt.expired(),
            "player chose"
        );
        self.core.apply_action(action);
        // Action committed: push snapshot reflecting the new state.
        self.core.push_snapshots();
        (action, prompt)
    }

    /// Transition to Showdown phase when hand is terminal.
    pub fn into_showdown(self) -> Engine<Showdown> {
        Engine {
            core: self.core,
            phase: PhantomData,
        }
    }

    async fn next_action(&mut self, pos: Position) -> (Action, Prompt) {
        loop {
            match self.poll_action(pos).await {
                Some((a, prompt)) if self.core.live.game().is_allowed(&a) => return (a, prompt),
                Some((a, _)) => tracing::warn!(seat = pos, action = %a, "action rejected"),
                None => {}
            }
        }
    }

    async fn poll_action(&mut self, pos: Position) -> Option<(Action, Prompt)> {
        match tokio::time::timeout(self.core.timing.decision, self.core.rx.recv()).await {
            Err(_) => {
                tracing::debug!(seat = pos, "player timed out");
                Some((self.core.live.game().passive(), Prompt::Expired))
            }
            Ok(inbox) => inbox.filter(|(j, _)| *j == pos).and_then(|(_, e)| match e {
                Event::Action(action) => Some((action, Prompt::Acted)),
                Event::Disconnect(p) => {
                    self.core.disconnected.insert(p);
                    Some((self.core.live.game().passive(), Prompt::Acted))
                }
                Event::Decision(_) => None,
            }),
        }
    }
}

/// Showdown phase: revealing cards and settling.
impl Engine<Showdown> {
    /// Returns true if this is a showdown (multiple players remain).
    pub fn is_showdown(&self) -> bool {
        self.core.live.game().is_showdown()
    }

    /// Executes showdown reveal sequence.
    pub async fn showdown(&mut self) {
        if !self.core.live.game().is_showdown() {
            for &p in &self.core.showoffs.clone() {
                self.reveal(p, true);
            }
            self.core.push_snapshots();
            if self.core.disconnected.is_empty() {
                tokio::select! {
                    biased;
                    () = self.core.skip.notified() => {},
                    () = tokio::time::sleep(self.core.timing.results) => {},
                }
            }
            return;
        }
        let forced = self.forced_reveals();
        let order = self.showdown_order();
        let mut showed = Vec::new();
        let deadline = tokio::time::Instant::now() + self.core.timing.showdown;
        loop {
            tokio::select! {
                biased;
                () = tokio::time::sleep_until(deadline) => break,
                () = self.core.skip.notified() => break,
                msg = self.core.rx.recv() => {
                    if let Some((pos, _)) = msg
                        && !showed.contains(&pos) && order.contains(&pos) {
                            self.reveal(pos, true);
                            self.core.push_snapshots();
                            showed.push(pos);
                        }
                }
            }
        }
        for p in forced.into_iter().filter(|p| !showed.contains(p)).collect::<Vec<_>>() {
            self.reveal(p, true);
            showed.push(p);
        }
        // Players who never showed are mucking — no event is needed for that.
        self.core.push_snapshots();
    }

    fn reveal(&mut self, seat: Position, show: bool) {
        if show {
            let hole = self.core.live.game().seats()[seat].cards();
            self.core.live.show(seat, hole);
        }
    }

    /// Apply settlement and push hand-end snapshot.
    pub fn settle(&mut self) {
        let settlements = self.core.live.game().settlements();
        tracing::debug!(?settlements, "settle");
        self.core.live.settle(settlements);
        self.core.record_hand_closing();
        self.core.push_snapshots();
    }

    /// Advance to next hand or finish.
    pub async fn conclude(mut self) -> Result<Engine<Dealing>, Engine<Finished>> {
        if let Some(next) = self.core.live.game().continuation() {
            self.core.live.start(self.core.live.epoch() + 1, next);
            self.core.commence().await;
            Ok(Engine {
                core: self.core,
                phase: PhantomData,
            })
        } else {
            tracing::info!("game over - a player is busted");
            Err(Engine {
                core: self.core,
                phase: PhantomData,
            })
        }
    }

    fn forced_reveals(&self) -> Vec<Position> {
        let settlements = self.core.live.game().settlements();
        self.core
            .live
            .game()
            .seats()
            .iter()
            .enumerate()
            .filter(|(_, seat)| seat.state() != State::Folding)
            .filter(|(i, seat)| {
                seat.state() == State::Shoving
                    || settlements.get(*i).is_some_and(|s| s.pnl().reward() > 0)
                    || self.core.showoffs.contains(i)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn showdown_order(&self) -> Vec<Position> {
        let n = self.core.live.game().n();
        let first = self
            .core
            .recall(0)
            .aggressor()
            .unwrap_or_else(|| (self.core.live.game().dealer().position() + 1) % n);
        (0..n)
            .map(|i| (first + i) % n)
            .filter(|i| self.core.live.game().seats()[*i].state().is_active())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_default_is_seating() {
        let engine = Engine::<Seating>::default();
        assert_eq!(engine.core.live.game().pot(), Game::sblind() + Game::bblind());
    }
}
