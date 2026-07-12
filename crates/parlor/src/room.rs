use super::*;
use bouncer::Member;
use bouncer::User;
use kicker::Reason;
use kicker::Turn;
use daybook::*;
use pokerkit::*;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

/// Live poker room coordinator.
/// Imperative shell that owns Engine (functional core) and handles
/// identity, user tracking, and persistence concerns.
pub struct Room {
    id: ID<Self>,
    db: Arc<Client>,
    stakes: Chips,
    context: HandContext,
    users: Vec<User>,
    idle: usize,
}

impl Room {
    pub fn new(id: ID<Self>, stakes: Chips, db: Arc<Client>) -> Self {
        Self {
            id,
            db,
            stakes,
            users: Vec::new(),
            context: HandContext::default(),
            idle: 0,
        }
    }

    pub fn stakes(&self) -> Chips {
        self.stakes
    }

    pub fn sit<P, U>(
        &mut self,
        engine: &mut Engine<Seating>,
        player: P,
        user: U,
        wire: Option<tokio::sync::mpsc::UnboundedSender<String>>,
    ) where
        P: Player + 'static,
        U: Into<User>,
    {
        engine.sit(player, wire);
        self.users.push(user.into());
    }
}

impl Room {
    #[tracing::instrument(skip_all, fields(room = %self.id))]
    pub async fn run(mut self, engine: Engine<Seating>, start: tokio::sync::oneshot::Receiver<()>) {
        tracing::debug!("waiting for player");
        if let Ok(Ok(())) = tokio::time::timeout(std::time::Duration::from_millis(PACE_ROOM_STARTUP), start).await {
        } else {
            tracing::warn!("startup timeout, no player connected");
            return;
        }
        tracing::debug!("starting game loop");
        let mut dealing = engine.start().await;
        loop {
            self.reset_hand(&dealing);
            let acted = self.play_hand(&mut dealing).await;
            let mut showdown = dealing.into_showdown();
            self.run_showdown(&mut showdown).await;
            self.flush_hand(&showdown).await;
            if self.should_stop(&showdown) {
                showdown.end_session(showdown.final_stacks(), Reason::Left);
                break;
            }
            self.idle = if acted { 0 } else { self.idle + 1 };
            if self.idle >= MAX_IDLE_HANDS {
                tracing::info!(idle = self.idle, "idle limit reached");
                showdown.end_session(showdown.final_stacks(), Reason::Idle);
                break;
            }
            tokio::select! {
                () = tokio::time::sleep(showdown.timing().results) => {},
                () = showdown.skip().notified() => {},
            }
            if self.should_stop(&showdown) {
                showdown.end_session(showdown.final_stacks(), Reason::Left);
                break;
            }
            match showdown.conclude().await {
                Ok(next) => dealing = next,
                Err(finished) => {
                    tracing::info!("game over");
                    finished.end_session(finished.final_stacks(), Reason::Busted);
                    break;
                }
            }
        }
    }
    /// Returns whether the human player actively decided at least once.
    async fn play_hand(&mut self, engine: &mut Engine<Dealing>) -> bool {
        let mut acted = false;
        loop {
            match engine.turn() {
                Turn::Chance => {
                    engine.deal().await;
                    if let Some(draw) = engine.last_draw() {
                        self.context.record(0, draw, None);
                    }
                }
                Turn::Choice(p) => {
                    let start = std::time::Instant::now();
                    let (action, prompt) = engine.ask(p).await;
                    self.context.record(p, action, Some(start.elapsed().as_millis() as i32));
                    if p == 0 && !prompt.expired() {
                        acted = true;
                    }
                }
                Turn::Terminal => break,
            }
        }
        acted
    }

    async fn run_showdown(&mut self, engine: &mut Engine<Showdown>) {
        engine.showdown().await;
        engine.settle();
    }

    fn should_stop<T>(&self, engine: &Engine<T>) -> bool {
        (0..self.users.len())
            .filter(|pos| engine.is_disconnected(*pos))
            .inspect(|pos| tracing::info!(seat = pos, "player disconnected"))
            .next()
            .is_some()
    }
}

impl Room {
    fn user(&self, pos: Position) -> Option<ID<Member>> {
        self.users.get(pos).and_then(User::id)
    }

    fn reset_hand(&mut self, engine: &Engine<Dealing>) {
        self.context = HandContext::new(engine.hand(), &engine.game());
    }

    async fn flush_hand(&mut self, engine: &Engine<Showdown>) {
        for (i, s) in engine.game().settlements().iter().enumerate() {
            self.context.set_pnl(i, s.won());
        }
        let hand = self
            .context
            .to_hand(self.id().cast(), engine.game().board(), engine.game().pot());
        self.db
            .create_hand(&hand)
            .await
            .inspect_err(|e| tracing::error!(error = %e, "failed to record hand"))
            .ok();
        for ref player in self.context.participants(hand.id(), |p| self.user(p)) {
            self.db
                .create_player(player)
                .await
                .inspect_err(|e| tracing::error!(error = %e, "failed to record player"))
                .ok();
        }
        for ref play in self.context.plays(hand.id(), |p| self.user(p)) {
            self.db
                .create_action(play)
                .await
                .inspect_err(|e| tracing::error!(error = %e, "failed to record action"))
                .ok();
        }
        tracing::info!(hand = %hand.id(), "recorded hand");
    }
}

impl Unique for Room {
    fn id(&self) -> ID<Self> {
        self.id
    }
}

impl Schema for Room {
    fn name() -> &'static str {
        rooms()
    }

    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[tokio_postgres::types::Type::UUID, tokio_postgres::types::Type::INT2]
    }

    fn creates() -> &'static str {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        SQL.get_or_init(|| {
            leaked(format!(
                "CREATE TABLE IF NOT EXISTS {} (
                id          UUID PRIMARY KEY,
                stakes      SMALLINT NOT NULL
            );",
                rooms()
            ))
        })
    }

    fn indices() -> &'static str {
        ""
    }

    fn copy() -> &'static str {
        unimplemented!()
    }

    fn truncates() -> &'static str {
        unimplemented!()
    }

    fn freeze() -> &'static str {
        unimplemented!()
    }
}
