//! Persistence of one session's hands into the shared hand-history tables.

use bouncer::*;
use kicker::*;
use parlor::AdversaryExt;
use parlor::*;
use pokerkit::*;
use std::sync::Arc;

/// Writes every hand of one benchmark session to `hands` / `players` / `plays`, under a room of
/// its own.
///
/// A session owns exactly one recorder and one room, so concurrent sessions never interleave
/// into the same room's hand numbering.
pub struct Recorder {
    db: Arc<tokio_postgres::Client>,
    room: ID<records::Room>,
    context: HandContext,
    hero: ID<Member>,
    rival: ID<Member>,
    seat: Position,
    hand: u64,
}

impl Recorder {
    /// Open a room for one session of `hero` against `rival`, staked at the
    /// service's stack depth in our chip scale.
    pub async fn new(db: Arc<tokio_postgres::Client>, hero: ID<Member>, rival: Adversary, stack: Chips) -> Self {
        let room = ID::<records::Room>::default();
        db.execute("INSERT INTO rooms (id, stakes) VALUES ($1, $2)", &[&room.inner(), &stack])
            .await
            .expect("failed to create room");
        Self {
            db,
            room,
            context: HandContext::default(),
            hero,
            rival: rival.id(),
            seat: 0,
            hand: 0,
        }
    }

    /// Start a new hand with hero in `seat`.
    pub fn begin(&mut self, game: &Game, seat: Position) {
        *self.context_mut() = HandContext::new(self.hand(), game);
        *self.seat_mut() = seat;
        *self.hand_mut() += 1;
    }

    /// Append one action. `elapsed` is hero's think-time in ms, `None` for
    /// the opponent (whose clock we can't see).
    pub fn record(&mut self, pos: Position, action: Action, elapsed: Option<i32>) {
        self.context_mut().record(pos, action, elapsed);
    }

    /// Set a seat's net chips for the hand, in our chip scale.
    pub fn pnl(&mut self, seat: Position, pnl: Chips) {
        self.context_mut().set_pnl(seat, pnl);
    }

    fn db(&self) -> &Arc<tokio_postgres::Client> {
        &self.db
    }

    fn room(&self) -> ID<records::Room> {
        self.room
    }

    fn context(&self) -> &HandContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut HandContext {
        &mut self.context
    }

    fn hero(&self) -> ID<Member> {
        self.hero
    }

    fn rival(&self) -> ID<Member> {
        self.rival
    }

    fn seat(&self) -> Position {
        self.seat
    }

    fn seat_mut(&mut self) -> &mut Position {
        &mut self.seat
    }

    fn hand(&self) -> u64 {
        self.hand
    }

    fn hand_mut(&mut self) -> &mut u64 {
        &mut self.hand
    }

    /// Write the finished hand out.
    pub async fn flush(&self, witness: &Witness, board: deuce::Board, pot: Chips) {
        let hand = self.context().to_hand(self.room(), board, pot);
        let seats = |p| Some(if p == self.seat() { self.hero() } else { self.rival() });
        self.db().create_hand(&hand).await.expect("failed to record hand");
        for ref player in self.context().participants(hand.id(), seats) {
            self.db().create_player(player).await.expect("failed to record player");
        }
        for ref play in self.context().plays(hand.id(), seats) {
            self.db().create_action(play).await.expect("failed to record action");
        }
        for line in format!("{witness}").lines() {
            tracing::trace!("{line}");
        }
    }
}
