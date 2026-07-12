use crate::records::Hand;
use crate::records::Participant;
use crate::records::Play;
use crate::records::Visibility;
use crate::room::Room;
use bouncer::*;
use deuce::Board;
use deuce::Hole;
use kicker::Action;
use daybook::*;
use pokerkit::*;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

/// Repository trait for hand history database operations.
#[allow(async_fn_in_trait)]
pub trait HistoryRepository {
    async fn create_room(&self, room: &Room) -> Result<(), PgErr>;
    async fn create_hand(&self, hand: &Hand) -> Result<(), PgErr>;
    async fn create_action(&self, action: &Play) -> Result<(), PgErr>;
    async fn create_player(&self, player: &Participant) -> Result<(), PgErr>;
    async fn update_visibility(&self, hand: ID<Hand>, user: ID<Member>, visibility: Visibility) -> Result<(), PgErr>;
    async fn get_hands(&self, user: ID<Member>, limit: i64) -> Result<Vec<ID<Hand>>, PgErr>;
    async fn get_hand(&self, hand: ID<Hand>) -> Result<Option<Hand>, PgErr>;
    async fn get_players(&self, hand: ID<Hand>) -> Result<Vec<Participant>, PgErr>;
    async fn get_actions(&self, hand: ID<Hand>) -> Result<Vec<Play>, PgErr>;
    async fn get_visible(&self, hand: ID<Hand>, seat: Position, viewer: ID<Member>) -> Result<Option<Hole>, PgErr>;
}

impl HistoryRepository for Arc<Client> {
    async fn create_room(&self, room: &Room) -> Result<(), PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("INSERT INTO {} (id, stakes) VALUES ($1, $2)", rooms()));
        self.execute(sql.as_str(), &[&room.id().inner(), &room.stakes()])
            .await
            .map(|_| ())
    }

    async fn create_hand(&self, hand: &Hand) -> Result<(), PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!("INSERT INTO {} (id, room_id, board, pot, dealer) VALUES ($1, $2, $3, $4, $5)", hands())
        });
        let board: deuce::Hand = hand.board().into();
        self.execute(
            sql.as_str(),
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
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "INSERT INTO {} (hand_id, user_id, seat, hole, stack, visibility, pnl) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            players()
        ));
        let hole: deuce::Hand = player.hole().into();
        let user_id: Option<uuid::Uuid> = player.user().map(|id| id.inner());
        self.execute(
            sql.as_str(),
            &[
                &player.hand().inner(),
                &user_id,
                &(player.seat() as i16),
                &(u64::from(hole) as i64),
                &player.stack(),
                &i16::from(player.visibility()),
                &player.pnl(),
            ],
        )
        .await
        .map(|_| ())
    }

    async fn create_action(&self, action: &Play) -> Result<(), PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "INSERT INTO {} (hand_id, seq, player_id, encoded, elapsed_ms) VALUES ($1, $2, $3, $4, $5)",
                actions()
            )
        });
        let player_id: Option<uuid::Uuid> = action.player().map(|id| id.inner());
        self.execute(
            sql.as_str(),
            &[
                &action.hand().inner(),
                &action.seq(),
                &player_id,
                &(u32::from(action.action()) as i32),
                &action.elapsed(),
            ],
        )
        .await
        .map(|_| ())
    }

    async fn update_visibility(&self, hand: ID<Hand>, user: ID<Member>, visibility: Visibility) -> Result<(), PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql =
            SQL.get_or_init(|| format!("UPDATE {} SET visibility = $3 WHERE hand_id = $1 AND user_id = $2", players()));
        self.execute(sql.as_str(), &[&hand.inner(), &user.inner(), &i16::from(visibility)])
            .await
            .map(|_| ())
    }

    async fn get_hands(&self, user: ID<Member>, limit: i64) -> Result<Vec<ID<Hand>>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT h.id FROM {} h JOIN {} p ON p.hand_id = h.id WHERE p.user_id = $1 ORDER BY h.id DESC LIMIT $2",
                hands(),
                players()
            )
        });
        self.query(sql.as_str(), &[&user.inner(), &limit])
            .await
            .map(|rows| rows.iter().map(|row| ID::from(row.get::<_, uuid::Uuid>(0))).collect())
    }

    async fn get_hand(&self, hand: ID<Hand>) -> Result<Option<Hand>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT id, room_id, board, pot, dealer FROM {} WHERE id = $1", hands()));
        self.query_opt(sql.as_str(), &[&hand.inner()]).await.map(|opt| {
            opt.map(|row| {
                Hand::new(
                    ID::from(row.get::<_, uuid::Uuid>(0)),
                    ID::from(row.get::<_, uuid::Uuid>(1)),
                    Board::from(deuce::Hand::from(row.get::<_, i64>(2) as u64)),
                    row.get::<_, Chips>(3),
                    row.get::<_, i16>(4) as Position,
                )
            })
        })
    }

    async fn get_players(&self, hand: ID<Hand>) -> Result<Vec<Participant>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT hand_id, user_id, seat, hole, stack, visibility, pnl FROM {} WHERE hand_id = $1 ORDER BY seat",
                players()
            )
        });
        self.query(sql.as_str(), &[&hand.inner()]).await.map(|rows| {
            rows.iter()
                .map(|row| {
                    let user_id: Option<uuid::Uuid> = row.get(1);
                    Participant::with_visibility(
                        ID::from(row.get::<_, uuid::Uuid>(0)),
                        user_id.map(ID::from),
                        row.get::<_, i16>(2) as Position,
                        Hole::from(deuce::Hand::from(row.get::<_, i64>(3) as u64)),
                        row.get::<_, Chips>(4),
                        Visibility::from(row.get::<_, i16>(5)),
                        row.get::<_, Chips>(6),
                    )
                })
                .collect()
        })
    }

    async fn get_actions(&self, hand: ID<Hand>) -> Result<Vec<Play>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT hand_id, seq, player_id, encoded, elapsed_ms FROM {} WHERE hand_id = $1 ORDER BY seq",
                actions()
            )
        });
        self.query(sql.as_str(), &[&hand.inner()]).await.map(|rows| {
            rows.iter()
                .map(|row| {
                    let player_id: Option<uuid::Uuid> = row.get(2);
                    Play::new(
                        ID::from(row.get::<_, uuid::Uuid>(0)),
                        row.get::<_, Epoch>(1),
                        player_id.map(ID::from),
                        Action::from(row.get::<_, i32>(3) as u32),
                        row.get::<_, Option<i32>>(4),
                    )
                })
                .collect()
        })
    }

    async fn get_visible(&self, hand: ID<Hand>, seat: Position, viewer: ID<Member>) -> Result<Option<Hole>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT hole FROM {} WHERE hand_id = $1 AND seat = $2 AND (user_id = $3 OR visibility = 1)",
                players()
            )
        });
        self.query_opt(sql.as_str(), &[&hand.inner(), &(seat as i16), &viewer.inner()])
            .await
            .map(|opt| opt.map(|row| Hole::from(deuce::Hand::from(row.get::<_, i64>(0) as u64))))
    }
}
