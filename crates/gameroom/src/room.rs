use super::*;
use rbp_auth::Member;
use rbp_auth::User;
use rbp_core::*;
use rbp_gameplay::Turn;
use rbp_database::*;
use std::sync::Arc;
use tokio_postgres::Client;

/// Live poker room coordinator.
/// Imperative shell that owns Engine (functional core) and handles
/// identity, user tracking, and persistence concerns.
pub struct Room {
    id: ID<Self>,
    db: Arc<Client>,
    stakes: Chips,
    engine: EngineState,
    context: HandContext,
    users: Vec<User>,
}

impl Room {
    pub fn new(id: ID<Self>, stakes: Chips, db: Arc<Client>) -> Self {
        Self {
            id,
            db,
            stakes,
            users: Vec::new(),
            engine: EngineState::default(),
            context: HandContext::default(),
        }
    }
    pub fn stakes(&self) -> Chips {
        self.stakes
    }
    pub fn sit<P, U>(&mut self, player: P, user: U)
    where
        P: Player + 'static,
        U: Into<User>,
    {
        self.engine.as_seating().sit(player);
        self.users.push(user.into());
    }
}

impl Room {
    pub async fn run(
        mut self,
        start: tokio::sync::oneshot::Receiver<()>,
        done: tokio::sync::oneshot::Sender<()>,
    ) {
        log::debug!("[room {}] waiting for player", self.id);
        let _ = start.await;
        log::debug!("[room {}] starting game loop", self.id);
        self.engine.start();
        loop {
            self.reset_hand();
            self.play_hand().await;
            self.engine.into_showdown();
            self.run_showdown().await;
            self.flush_hand().await;
            if self.should_stop() {
                break;
            }
            self.engine.conclude();
            if self.engine.is_finished() {
                log::info!("[room {}] game over", self.id);
                break;
            }
        }
        let _ = done.send(());
    }
    async fn play_hand(&mut self) {
        let engine = match &mut self.engine {
            EngineState::Dealing(e) => e,
            _ => panic!("play_hand called in wrong phase"),
        };
        loop {
            match engine.turn() {
                Turn::Chance => engine.deal().await,
                Turn::Choice(p) => {
                    let action = engine.ask(p).await;
                    self.context.record(p, action);
                }
                Turn::Terminal => break,
            }
        }
    }
    async fn run_showdown(&mut self) {
        let engine = match &mut self.engine {
            EngineState::Showdown(e) => e,
            _ => panic!("run_showdown called in wrong phase"),
        };
        engine.showdown().await;
        engine.settle();
    }
    fn should_stop(&self) -> bool {
        match &self.engine {
            EngineState::Showdown(e) => e.human_disconnected(),
            _ => false,
        }
    }
}

impl Room {
    fn user(&self, pos: Position) -> Option<ID<Member>> {
        self.users.get(pos).and_then(User::id)
    }
    fn reset_hand(&mut self) {
        let (hand_number, game) = match &self.engine {
            EngineState::Dealing(e) => (e.hand(), e.game()),
            _ => panic!("reset_hand called in wrong phase"),
        };
        self.context = HandContext::new(hand_number, game);
    }
    async fn flush_hand(&self) {
        let game = match &self.engine {
            EngineState::Showdown(e) => e.game(),
            _ => panic!("flush_hand called in wrong phase"),
        };
        let hand = self.context.to_hand(self.id().cast(), game.board(), game.pot());
        self.db
            .create_hand(&hand)
            .await
            .expect("failed to record hand");
        for ref player in self.context.participants(hand.id(), |p| self.user(p)) {
            self.db
                .create_player(player)
                .await
                .expect("failed to record player");
        }
        for ref play in self.context.plays(hand.id(), |p| self.user(p)) {
            self.db
                .create_action(play)
                .await
                .expect("failed to record action");
        }
        log::info!("recorded hand {}", hand.id());
    }
}

impl Unique for Room {
    fn id(&self) -> ID<Self> {
        self.id
    }
}

impl Schema for Room {
    fn name() -> &'static str {
        ROOMS
    }
    fn columns() -> &'static [tokio_postgres::types::Type] {
        &[
            tokio_postgres::types::Type::UUID,
            tokio_postgres::types::Type::INT2,
        ]
    }
    fn creates() -> &'static str {
        const_format::concatcp!(
            "CREATE TABLE IF NOT EXISTS ",
            ROOMS,
            " (
                id          UUID PRIMARY KEY,
                stakes      SMALLINT NOT NULL
            );"
        )
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
