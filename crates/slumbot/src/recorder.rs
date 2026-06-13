use crate::translate::SLUMBOT_STACK;
use rbp_auth::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_gameroom::slumbot_opponent_id;
use rbp_gameroom::*;
use std::sync::Arc;

pub struct Recorder {
    db: Arc<tokio_postgres::Client>,
    room: ID<records::Room>,
    context: HandContext,
    hero: ID<Member>,
    villain: ID<Member>,
    seat: Position,
    hand: u64,
}

impl Recorder {
    pub async fn new(db: Arc<tokio_postgres::Client>, hero: ID<Member>) -> Self {
        let room = ID::<records::Room>::default();
        db.execute("INSERT INTO rooms (id, stakes) VALUES ($1, $2)", &[&room.inner(), &{ SLUMBOT_STACK }])
            .await
            .expect("failed to create room");
        Self {
            db,
            room,
            context: HandContext::default(),
            hero,
            villain: slumbot_opponent_id(),
            seat: 0,
            hand: 0,
        }
    }

    pub fn begin(&mut self, game: &Game, seat: Position) {
        self.context = HandContext::new(self.hand, game);
        self.seat = seat;
        self.hand += 1;
    }

    pub fn record(&mut self, pos: Position, action: Action, elapsed: Option<i32>) {
        self.context.record(pos, action, elapsed);
    }

    pub fn set_pnl(&mut self, seat: Position, pnl: Chips) {
        self.context.set_pnl(seat, pnl);
    }

    pub async fn flush(&self, witness: &Witness, board: rbp_cards::Board, pot: Chips) {
        let hand = self.context.to_hand(self.room, board, pot);
        let hero = self.hero;
        let villain = self.villain;
        let seat = self.seat;
        self.db.create_hand(&hand).await.expect("failed to record hand");
        for ref player in self
            .context
            .participants(hand.id(), |p| Some(if p == seat { hero } else { villain }))
        {
            self.db.create_player(player).await.expect("failed to record player");
        }
        for ref play in self
            .context
            .plays(hand.id(), |p| Some(if p == seat { hero } else { villain }))
        {
            self.db.create_action(play).await.expect("failed to record action");
        }
        for line in format!("{witness}").lines() {
            tracing::trace!("{line}");
        }
    }
}
