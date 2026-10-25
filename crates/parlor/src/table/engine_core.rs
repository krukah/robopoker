use super::*;
use kicker::*;
use pokerkit::*;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::mpsc::*;

/// Shared state across all engine phases.
pub(crate) struct EngineCore {
    live: LiveGame,
    timing: Pacing,
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
    pub(crate) fn new(tx: UnboundedSender<Inbox>, rx: UnboundedReceiver<Inbox>) -> Self {
        Self {
            live: LiveGame::default(),
            timing: Pacing::default(),
            skip: Arc::new(Notify::new()),
            tx,
            rx,
            players: Vec::new(),
            wires: Vec::new(),
            history: Vec::new(),
            disconnected: HashSet::new(),
            showoffs: HashSet::new(),
        }
    }

    pub(crate) async fn interruptible(&self, duration: std::time::Duration) {
        tokio::select! {
            biased;
            () = self.skip().notified() => {},
            () = tokio::time::sleep(duration) => {},
        }
    }

    pub(crate) fn apply_action(&mut self, action: Action) {
        self.live_mut().act(action);
    }

    pub(crate) fn recall(&self, pos: Position) -> Witness {
        let cards = self
            .live()
            .game()
            .seats()
            .get(pos)
            .unwrap()
            .cards()
            .into_iter()
            .chain(self.live().dealt())
            .collect::<Vec<_>>();
        self.live().actions().iter().copied().filter(Action::is_choice).fold(
            Witness::initial_with(
                Turn::Choice(pos),
                Arrangement::from(cards),
                self.live().root().buyins(),
                self.live().root().dealer().position(),
            ),
            |r, a| r.push(a),
        )
    }

    fn snapshot(&self, pos: Position) -> Snapshot {
        Snapshot {
            hand: self.live().epoch(),
            phase: self.live().phase(),
            witness: self.recall(pos),
            reveals: (0..self.live().holes().len())
                .map(|i| if i == pos { None } else { self.live().shown(i) })
                .collect(),
            settlements: self.live().settlements().to_vec(),
            history: self.history().clone(),
        }
    }

    pub(crate) fn push_snapshots(&self) {
        for pos in 0..self.wires().len() {
            self.send_wire(pos, ServerMessage::Snapshot(self.snapshot(pos)));
        }
    }

    pub(crate) fn push_session_end(&self, stacks: [Chips; N], reason: Reason) {
        for pos in 0..self.wires().len() {
            self.send_wire(pos, ServerMessage::session_end(stacks, reason));
        }
    }

    fn send_wire(&self, pos: Position, msg: ServerMessage) {
        if let Some(tx) = self.wires().get(pos).and_then(Option::as_ref)
            && let Err(e) = tx.send(msg.to_json())
        {
            tracing::warn!(seat = pos, error = ?e, "wire send failed");
        }
    }

    pub(crate) fn record_hand_closing(&mut self) {
        let hand = CompletedHand {
            epoch: self.live().epoch(),
            settlements: self.live().settlements().to_vec(),
        };
        self.history_mut().push(hand);
    }

    pub(crate) async fn commence(&mut self) {
        self.interruptible(self.timing().hole()).await;
        for i in 0..self.players().len() {
            let hole = self.live().game().seats()[i].cards();
            self.live_mut().deal_hole(i, hole);
        }
        self.push_snapshots();
    }

    pub(crate) fn unicast(&self, i: usize, event: Event) {
        tracing::trace!(seat = i, %event, "unicast");
        match self.players().get(i).map(|inbox| inbox.send(event)) {
            Some(Ok(())) => {}
            Some(Err(e)) => tracing::warn!(seat = i, error = ?e, "unicast failed"),
            None => tracing::warn!(seat = i, "unicast: no such player"),
        }
    }
}
impl Recall for EngineCore {
    fn root(&self) -> Game {
        *self.live().root()
    }

    fn actions(&self) -> &[Action] {
        self.live().actions()
    }
}

impl EngineCore {
    pub fn live_mut(&mut self) -> &mut LiveGame {
        &mut self.live
    }

    pub fn live(&self) -> &LiveGame {
        &self.live
    }

    fn history(&self) -> &Vec<CompletedHand> {
        &self.history
    }

    fn history_mut(&mut self) -> &mut Vec<CompletedHand> {
        &mut self.history
    }

    pub fn timing(&self) -> Pacing {
        self.timing
    }

    pub fn skip_mut(&mut self) -> &mut Arc<Notify> {
        &mut self.skip
    }

    pub fn skip(&self) -> &Arc<Notify> {
        &self.skip
    }

    pub fn tx(&self) -> &UnboundedSender<Inbox> {
        &self.tx
    }

    pub fn rx_mut(&mut self) -> &mut UnboundedReceiver<Inbox> {
        &mut self.rx
    }

    pub fn players_mut(&mut self) -> &mut Vec<UnboundedSender<Event>> {
        &mut self.players
    }

    pub fn players(&self) -> &Vec<UnboundedSender<Event>> {
        &self.players
    }

    pub fn wires(&self) -> &Vec<Option<UnboundedSender<String>>> {
        &self.wires
    }

    pub fn wires_mut(&mut self) -> &mut Vec<Option<UnboundedSender<String>>> {
        &mut self.wires
    }

    pub fn disconnected_mut(&mut self) -> &mut HashSet<usize> {
        &mut self.disconnected
    }

    pub fn disconnected(&self) -> &HashSet<usize> {
        &self.disconnected
    }

    pub fn showoffs_mut(&mut self) -> &mut HashSet<usize> {
        &mut self.showoffs
    }

    pub fn showoffs(&self) -> &HashSet<usize> {
        &self.showoffs
    }
}
