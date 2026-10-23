use super::*;
use crate::Showdown;
use kicker::*;
use pokerkit::*;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::mpsc::*;

pub(crate) type Inbox = (usize, Event);

/// Functional core for a live poker game: state, rules, and player
/// communication. Driven by Room (the imperative shell), which owns
/// persistence. Phase transitions are encoded as typestate.
pub struct Engine<Phase> {
    core: EngineCore,
    phase: PhantomData<Phase>,
}

impl Default for Engine<Seating> {
    fn default() -> Self {
        Self {
            core: {
                let (tx, rx) = unbounded_channel();
                EngineCore::new(tx, rx)
            },
            phase: PhantomData,
        }
    }
}

impl<T> Recall for Engine<T> {
    fn root(&self) -> Game {
        *self.core().live().root()
    }

    fn actions(&self) -> &[Action] {
        self.core().live().actions()
    }
}

/// Phase-agnostic accessors available in any phase.
impl<T> Engine<T> {
    fn core(&self) -> &EngineCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut EngineCore {
        &mut self.core
    }

    fn release(self) -> EngineCore {
        self.core
    }

    fn shift<Q>(self) -> Engine<Q> {
        Engine {
            core: self.release(),
            phase: PhantomData,
        }
    }

    pub fn game(&self) -> Game {
        self.core().live().game()
    }

    pub fn hand(&self) -> u64 {
        self.core().live().epoch()
    }

    pub fn timing(&self) -> Pacing {
        self.core().timing()
    }

    pub fn skip(&self) -> &Arc<Notify> {
        self.core().skip()
    }

    pub fn is_disconnected(&self, pos: usize) -> bool {
        self.core().disconnected().contains(&pos)
    }

    pub fn final_stacks(&self) -> [Chips; N] {
        let game = self.core().live().game();
        let settlements = game.settlements();
        let seats = game.seats();
        std::array::from_fn(|i| seats[i].stack() + settlements[i].pnl().reward())
    }

    pub fn end_session(&self, stacks: [Chips; N], reason: Reason) {
        self.core().push_session_end(stacks, reason);
    }
}

/// Seating phase: accepting players.
impl Engine<Seating> {
    pub fn set_skip(&mut self, skip: Arc<Notify>) {
        *self.core_mut().skip_mut() = skip;
    }

    pub fn sit<P>(&mut self, player: P, wire: Option<UnboundedSender<String>>)
    where
        P: Player + 'static,
    {
        let pos = self.core().players().len();
        if player.shows() {
            self.core_mut().showoffs_mut().insert(pos);
        }
        let tx = self.core().tx().clone();
        self.core_mut()
            .players_mut()
            .push(Actor::spawn(pos, Box::new(player), tx));
        self.core_mut().wires_mut().push(wire);
    }

    /// Transition to Dealing phase. Broadcasts hand start and hole cards.
    pub async fn start(mut self) -> Engine<Dealing> {
        self.core_mut().commence().await;
        self.shift()
    }
}

/// Dealing phase: hand in progress.
impl Engine<Dealing> {
    pub fn turn(&self) -> Turn {
        self.core().live().game().turn()
    }

    /// Returns the most recent Draw action, if the last action was a draw.
    pub fn last_draw(&self) -> Option<Action> {
        self.core().live().actions().last().copied().filter(Action::is_chance)
    }

    /// Deal next community cards (chance node).
    pub async fn deal(&mut self) {
        let hand = self.core().live().game().reveal(&mut rand::rng()).hand().unwrap();
        tracing::debug!(?hand, "dealing");
        self.core_mut().live_mut().deal(hand);
        self.core().push_snapshots();
        self.core().interruptible(self.core().timing().board()).await;
    }

    /// Get decision from player at position (choice node).
    /// Returns the action taken and how it was prompted.
    pub async fn ask(&mut self, pos: Position) -> (Action, Prompt) {
        tracing::debug!(seat = pos, "asking for action");
        // Tell the player actor it's their turn so internal Players can decide.
        self.core().unicast(pos, Event::Decision(self.core().recall(pos)));
        // Push fresh snapshot so the wire learns who to_act is + sees legal moves.
        self.core().push_snapshots();
        let (action, prompt) = self.next_action(pos).await;
        // A bot answers instantly, which reads as inhuman. A human seat
        // already has its decision window and needs no pause.
        if self.core().wires().get(pos).is_none_or(Option::is_none) {
            self.core().interruptible(self.core().timing().think()).await;
        }
        tracing::debug!(
            seat = pos,
            %action,
            timeout = prompt.expired(),
            "player chose"
        );
        self.core_mut().apply_action(action);
        // Action committed: push snapshot reflecting the new state.
        self.core().push_snapshots();
        (action, prompt)
    }

    /// Transition to Showdown phase when hand is terminal.
    pub fn into_showdown(self) -> Engine<Showdown> {
        self.shift()
    }

    async fn next_action(&mut self, pos: Position) -> (Action, Prompt) {
        loop {
            match self.poll_action(pos).await {
                Some((a, prompt)) if self.core().live().game().is_allowed(&a) => return (a, prompt),
                Some((a, _)) => tracing::warn!(seat = pos, action = %a, "action rejected"),
                None => {}
            }
        }
    }

    async fn poll_action(&mut self, pos: Position) -> Option<(Action, Prompt)> {
        match tokio::time::timeout(self.core().timing().decision(), self.core_mut().rx_mut().recv()).await {
            Err(_) => {
                tracing::debug!(seat = pos, "player timed out");
                Some((self.core().live().game().passive(), Prompt::Expired))
            }
            Ok(inbox) => inbox.filter(|(j, _)| *j == pos).and_then(|(_, e)| match e {
                Event::Action(action) => Some((action, Prompt::Acted)),
                Event::Disconnect(p) => {
                    self.core_mut().disconnected_mut().insert(p);
                    Some((self.core().live().game().passive(), Prompt::Acted))
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
        self.core().live().game().is_showdown()
    }

    /// Executes showdown reveal sequence.
    pub async fn showdown(&mut self) {
        if !self.core().live().game().is_showdown() {
            for &p in &self.core().showoffs().clone() {
                self.reveal(p, true);
            }
            self.core().push_snapshots();
            if self.core().disconnected().is_empty() {
                tokio::select! {
                    biased;
                    () = self.core().skip().notified() => {},
                    () = tokio::time::sleep(self.core().timing().results()) => {},
                }
            }
            return;
        }
        let forced = self.forced_reveals();
        let order = self.showdown_order();
        let mut showed = Vec::new();
        let deadline = tokio::time::Instant::now() + self.core().timing().reveal();
        let skip = self.core().skip().clone();
        loop {
            tokio::select! {
                biased;
                () = tokio::time::sleep_until(deadline) => break,
                () = skip.notified() => break,
                msg = self.core_mut().rx_mut().recv() => {
                    if let Some((pos, _)) = msg
                        && !showed.contains(&pos) && order.contains(&pos) {
                            self.reveal(pos, true);
                            self.core().push_snapshots();
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
        self.core().push_snapshots();
    }

    fn reveal(&mut self, seat: Position, show: bool) {
        if show {
            let hole = self.core().live().game().seats()[seat].cards();
            self.core_mut().live_mut().show(seat, hole);
        }
    }

    /// Apply settlement and push hand-end snapshot.
    pub fn settle(&mut self) {
        let settlements = self.core().live().game().settlements();
        tracing::debug!(?settlements, "settle");
        self.core_mut().live_mut().settle(settlements);
        self.core_mut().record_hand_closing();
        self.core().push_snapshots();
    }

    /// Advance to next hand or finish.
    ///
    /// The `Err` arm is a typestate branch, not a failure — boxing it to satisfy
    /// `result_large_err` would buy an allocation per hand and say the wrong
    /// thing about what `Finished` means.
    #[allow(clippy::result_large_err)]
    pub async fn conclude(mut self) -> Result<Engine<Dealing>, Engine<Finished>> {
        if let Some(next) = self.core().live().game().continuation() {
            let epoch = self.core().live().epoch() + 1;
            self.core_mut().live_mut().start(epoch, next);
            self.core_mut().commence().await;
            Ok(self.shift())
        } else {
            tracing::info!("game over - a player is busted");
            Err(self.shift())
        }
    }

    fn forced_reveals(&self) -> Vec<Position> {
        let settlements = self.core().live().game().settlements();
        self.core()
            .live()
            .game()
            .seats()
            .iter()
            .enumerate()
            .filter(|(_, seat)| seat.state() != State::Folding)
            .filter(|(i, seat)| {
                seat.state() == State::Shoving
                    || settlements.get(*i).is_some_and(|s| s.pnl().reward() > 0)
                    || self.core().showoffs().contains(i)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn showdown_order(&self) -> Vec<Position> {
        let n = self.core().live().game().n();
        let first = self
            .core()
            .recall(0)
            .aggressor()
            .unwrap_or_else(|| (self.core().live().game().dealer().position() + 1) % n);
        (0..n)
            .map(|i| (first + i) % n)
            .filter(|i| self.core().live().game().seats()[*i].state().is_active())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_default_is_seating() {
        let engine = Engine::<Seating>::default();
        assert_eq!(engine.core().live().game().pot(), Game::sblind() + Game::bblind());
    }
}
