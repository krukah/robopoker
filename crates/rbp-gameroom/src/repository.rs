use super::Room;
use rbp_auth::*;
use rbp_cards::Board;
use rbp_cards::Hole;
use rbp_core::*;
use rbp_gameplay::Action;
use rbp_pg::*;
use rbp_records::Hand;
use rbp_records::Participant;
use rbp_records::Play;
use std::sync::Arc;
use tokio_postgres::Client;

/// Repository trait for hand history database operations.
#[allow(async_fn_in_trait)]
pub trait HistoryRepository {
    async fn create_room(&self, room: &Room) -> Result<(), PgErr>;
    async fn create_hand(&self, hand: &Hand) -> Result<(), PgErr>;
    async fn create_action(&self, action: &Play) -> Result<(), PgErr>;
    async fn create_player(&self, player: &Participant) -> Result<(), PgErr>;
    async fn update_showed(&self, hand: ID<Hand>, user: ID<Member>) -> Result<(), PgErr>;
    async fn update_mucked(&self, hand: ID<Hand>, user: ID<Member>) -> Result<(), PgErr>;
    async fn get_hands(&self, user: ID<Member>, limit: i64) -> Result<Vec<ID<Hand>>, PgErr>;
    async fn get_hand(&self, hand: ID<Hand>) -> Result<Option<Hand>, PgErr>;
    async fn get_players(&self, hand: ID<Hand>) -> Result<Vec<Participant>, PgErr>;
    async fn get_actions(&self, hand: ID<Hand>) -> Result<Vec<Play>, PgErr>;
    async fn get_visible(
        &self,
        hand: ID<Hand>,
        seat: Position,
        viewer: ID<Member>,
    ) -> Result<Option<Hole>, PgErr>;
}

impl HistoryRepository for Arc<Client> {
    async fn create_room(&self, room: &Room) -> Result<(), PgErr> {
        self.execute(
            const_format::concatcp!("INSERT INTO ", ROOMS, " (id, stakes) VALUES ($1, $2)"),
            &[&room.id().inner(), &room.stakes()],
        )
        .await
        .map(|_| ())
    }
    async fn create_hand(&self, hand: &Hand) -> Result<(), PgErr> {
        let board: rbp_cards::Hand = hand.board().into();
        self.execute(
            const_format::concatcp!(
                "INSERT INTO ",
                HANDS,
                " (id, room_id, board, pot, dealer) VALUES ($1, $2, $3, $4, $5)"
            ),
            &[
                &hand.id().inner(),
                &hand.room().inner(),
                &(u64::from(board) as i64),
                &hand.pot(),
                &(hand.dealer() as i16),
            ],
        )
        .await
        .map(|_| ())
    }
    async fn create_player(&self, player: &Participant) -> Result<(), PgErr> {
        let hole: rbp_cards::Hand = player.hole().into();
        let user_id: Option<uuid::Uuid> = player.user().map(|id| id.inner());
        self.execute(
            const_format::concatcp!(
                "INSERT INTO ",
                PLAYERS,
                " (hand_id, user_id, seat, hole, stack, showed, mucked) VALUES ($1, $2, $3, $4, $5, $6, $7)"
            ),
            &[
                &player.hand().inner(),
                &user_id,
                &(player.seat() as i16),
                &(u64::from(hole) as i64),
                &player.stack(),
                &player.showed(),
                &player.mucked(),
            ],
        )
        .await
        .map(|_| ())
    }
    async fn create_action(&self, action: &Play) -> Result<(), PgErr> {
        let player_id: Option<uuid::Uuid> = action.player().map(|id| id.inner());
        self.execute(
            const_format::concatcp!(
                "INSERT INTO ",
                ACTIONS,
                " (hand_id, seq, player_id, encoded) VALUES ($1, $2, $3, $4)"
            ),
            &[
                &action.hand().inner(),
                &action.seq(),
                &player_id,
                &(u32::from(action.action()) as i32),
            ],
        )
        .await
        .map(|_| ())
    }
    async fn update_showed(&self, hand: ID<Hand>, user: ID<Member>) -> Result<(), PgErr> {
        self.execute(
            const_format::concatcp!(
                "UPDATE ",
                PLAYERS,
                " SET showed = TRUE WHERE hand_id = $1 AND user_id = $2"
            ),
            &[&hand.inner(), &user.inner()],
        )
        .await
        .map(|_| ())
    }
    async fn update_mucked(&self, hand: ID<Hand>, user: ID<Member>) -> Result<(), PgErr> {
        self.execute(
            const_format::concatcp!(
                "UPDATE ",
                PLAYERS,
                " SET mucked = TRUE WHERE hand_id = $1 AND user_id = $2"
            ),
            &[&hand.inner(), &user.inner()],
        )
        .await
        .map(|_| ())
    }
    async fn get_hands(&self, user: ID<Member>, limit: i64) -> Result<Vec<ID<Hand>>, PgErr> {
        self.query(
            const_format::concatcp!(
                "SELECT h.id FROM ",
                HANDS,
                " h JOIN ",
                PLAYERS,
                " p ON p.hand_id = h.id WHERE p.user_id = $1 ORDER BY h.id DESC LIMIT $2"
            ),
            &[&user.inner(), &limit],
        )
        .await
        .map(|rows| {
            rows.iter()
                .map(|row| ID::from(row.get::<_, uuid::Uuid>(0)))
                .collect()
        })
    }
    async fn get_hand(&self, hand: ID<Hand>) -> Result<Option<Hand>, PgErr> {
        self.query_opt(
            const_format::concatcp!(
                "SELECT id, room_id, board, pot, dealer FROM ",
                HANDS,
                " WHERE id = $1"
            ),
            &[&hand.inner()],
        )
        .await
        .map(|opt| {
            opt.map(|row| {
                Hand::new(
                    ID::from(row.get::<_, uuid::Uuid>(0)),
                    ID::from(row.get::<_, uuid::Uuid>(1)),
                    Board::from(rbp_cards::Hand::from(row.get::<_, i64>(2) as u64)),
                    row.get::<_, Chips>(3),
                    row.get::<_, i16>(4) as Position,
                )
            })
        })
    }
    async fn get_players(&self, hand: ID<Hand>) -> Result<Vec<Participant>, PgErr> {
        self.query(
            const_format::concatcp!(
                "SELECT hand_id, user_id, seat, hole, stack, showed, mucked FROM ",
                PLAYERS,
                " WHERE hand_id = $1 ORDER BY seat"
            ),
            &[&hand.inner()],
        )
        .await
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    let user_id: Option<uuid::Uuid> = row.get(1);
                    let mut player = Participant::new(
                        ID::from(row.get::<_, uuid::Uuid>(0)),
                        user_id.map(ID::from),
                        row.get::<_, i16>(2) as Position,
                        Hole::from(rbp_cards::Hand::from(row.get::<_, i64>(3) as u64)),
                        row.get::<_, Chips>(4),
                    );
                    if row.get::<_, bool>(5) {
                        player.show();
                    }
                    if row.get::<_, bool>(6) {
                        player.muck();
                    }
                    player
                })
                .collect()
        })
    }
    async fn get_actions(&self, hand: ID<Hand>) -> Result<Vec<Play>, PgErr> {
        self.query(
            const_format::concatcp!(
                "SELECT hand_id, seq, player_id, encoded FROM ",
                ACTIONS,
                " WHERE hand_id = $1 ORDER BY seq"
            ),
            &[&hand.inner()],
        )
        .await
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    let player_id: Option<uuid::Uuid> = row.get(2);
                    Play::new(
                        ID::from(row.get::<_, uuid::Uuid>(0)),
                        row.get::<_, Epoch>(1),
                        player_id.map(ID::from),
                        Action::from(row.get::<_, i32>(3) as u32),
                    )
                })
                .collect()
        })
    }
    async fn get_visible(
        &self,
        hand: ID<Hand>,
        seat: Position,
        viewer: ID<Member>,
    ) -> Result<Option<Hole>, PgErr> {
        self.query_opt(
            const_format::concatcp!(
                "SELECT hole FROM ",
                PLAYERS,
                " WHERE hand_id = $1 AND seat = $2 AND (user_id = $3 OR showed = TRUE)"
            ),
            &[&hand.inner(), &(seat as i16), &viewer.inner()],
        )
        .await
        .map(|opt| {
            opt.map(|row| Hole::from(rbp_cards::Hand::from(row.get::<_, i64>(0) as u64)))
        })
    }
}
