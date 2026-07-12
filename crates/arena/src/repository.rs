use bouncer::Member;
use deuce::*;
use kicker::Action;
use daybook::*;
use parlor::records::{Hand as HandRecord, Participant, Play, Visibility};
use pokerkit::*;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio_postgres::Client;

/// Bulk data access for evaluation queries.
#[allow(async_fn_in_trait)]
pub trait EvaluationRepository {
    async fn eval_hands(&self, user: ID<Member>, limit: i64, offset: i64) -> Result<Vec<ID<HandRecord>>, PgErr>;
    async fn eval_bundle(&self, hand: ID<HandRecord>) -> Result<(HandRecord, Vec<Participant>, Vec<Play>), PgErr>;
    async fn eval_bundles(
        &self,
        hands: &[ID<HandRecord>],
    ) -> Result<Vec<(HandRecord, Vec<Participant>, Vec<Play>)>, PgErr>;
    async fn eval_hands_against(
        &self,
        user: ID<Member>,
        against: ID<Member>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ID<HandRecord>>, PgErr>;
    async fn eval_hands_by_stakes(
        &self,
        user: ID<Member>,
        stakes: i16,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ID<HandRecord>>, PgErr>;
    async fn eval_pnl(&self, user: ID<Member>, limit: i64, offset: i64) -> Result<Vec<(Chips, Chips)>, PgErr>;
    async fn eval_pnl_against(
        &self,
        user: ID<Member>,
        against: ID<Member>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Chips, Chips)>, PgErr>;
    async fn eval_pnl_by_stakes(
        &self,
        user: ID<Member>,
        stakes: i16,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Chips, Chips)>, PgErr>;
    async fn eval_pnl_human_hero(
        &self,
        bots: &[uuid::Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Chips, Chips)>, PgErr>;
    async fn eval_pnl_human_against(
        &self,
        user: ID<Member>,
        bots: &[uuid::Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Chips, Chips)>, PgErr>;
    async fn eval_hands_human_hero(
        &self,
        bots: &[uuid::Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ID<HandRecord>>, PgErr>;
    async fn eval_hands_human_against(
        &self,
        user: ID<Member>,
        bots: &[uuid::Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ID<HandRecord>>, PgErr>;
    async fn eval_count(&self, user: ID<Member>) -> Result<i64, PgErr>;
    async fn eval_count_against(&self, user: ID<Member>, against: ID<Member>) -> Result<i64, PgErr>;
    async fn eval_count_by_stakes(&self, user: ID<Member>, stakes: i16) -> Result<i64, PgErr>;
    async fn eval_count_human_hero(&self, bots: &[uuid::Uuid]) -> Result<i64, PgErr>;
    async fn eval_count_human_against(&self, user: ID<Member>, bots: &[uuid::Uuid]) -> Result<i64, PgErr>;
    async fn eval_policy(&self, past: i64, present: i16, choices: i64) -> Result<Vec<(i64, f32, f32)>, PgErr>;
    async fn eval_abstraction(&self, iso: i64) -> Result<Option<i16>, PgErr>;
    async fn eval_abstractions(&self, isos: &[i64]) -> Result<Vec<(i64, i16)>, PgErr>;
    async fn eval_chance_correction(
        &self,
        isos: &[i64],
        past: i64,
        choices: i64,
        observed_iso: i64,
    ) -> Result<Option<(f32, f32)>, PgErr>;
}

impl EvaluationRepository for Arc<Client> {
    async fn eval_hands(&self, user: ID<Member>, limit: i64, offset: i64) -> Result<Vec<ID<HandRecord>>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT h.id FROM {} h JOIN {} p ON p.hand_id = h.id WHERE p.user_id = $1 ORDER BY h.id DESC LIMIT $2 OFFSET $3",
            hands(), players()
        ));
        self.query(sql.as_str(), &[&user.inner(), &limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| ID::from(r.get::<_, uuid::Uuid>(0))).collect())
    }

    async fn eval_bundle(&self, hand: ID<HandRecord>) -> Result<(HandRecord, Vec<Participant>, Vec<Play>), PgErr> {
        static SQL_H: OnceLock<String> = OnceLock::<String>::new();
        static SQL_P: OnceLock<String> = OnceLock::<String>::new();
        static SQL_A: OnceLock<String> = OnceLock::<String>::new();
        let sql_h =
            SQL_H.get_or_init(|| format!("SELECT id, room_id, board, pot, dealer FROM {} WHERE id = $1", hands()));
        let sql_p = SQL_P.get_or_init(|| {
            format!(
                "SELECT hand_id, user_id, seat, hole, stack, visibility, pnl FROM {} WHERE hand_id = $1 ORDER BY seat",
                players()
            )
        });
        let sql_a = SQL_A.get_or_init(|| {
            format!(
                "SELECT hand_id, seq, player_id, encoded, elapsed_ms FROM {} WHERE hand_id = $1 ORDER BY seq",
                actions()
            )
        });
        let record = self
            .query_one(sql_h.as_str(), &[&hand.inner()])
            .await
            .map(|r| hand_from(&r))?;
        let players = self
            .query(sql_p.as_str(), &[&hand.inner()])
            .await
            .map(|rows| rows.iter().map(participant_from).collect())?;
        let actions = self
            .query(sql_a.as_str(), &[&hand.inner()])
            .await
            .map(|rows| rows.iter().map(play_from).collect())?;
        Ok((record, players, actions))
    }

    async fn eval_bundles(
        &self,
        hands: &[ID<HandRecord>],
    ) -> Result<Vec<(HandRecord, Vec<Participant>, Vec<Play>)>, PgErr> {
        static SQL_H: OnceLock<String> = OnceLock::<String>::new();
        static SQL_P: OnceLock<String> = OnceLock::<String>::new();
        static SQL_A: OnceLock<String> = OnceLock::<String>::new();
        let sql_h = SQL_H.get_or_init(|| {
            format!("SELECT id, room_id, board, pot, dealer FROM {} WHERE id = ANY($1) ORDER BY id", daybook::hands())
        });
        let sql_p = SQL_P.get_or_init(|| format!(
            "SELECT hand_id, user_id, seat, hole, stack, visibility, pnl FROM {} WHERE hand_id = ANY($1) ORDER BY hand_id, seat",
            daybook::players()
        ));
        let sql_a = SQL_A.get_or_init(|| format!(
            "SELECT hand_id, seq, player_id, encoded, elapsed_ms FROM {} WHERE hand_id = ANY($1) ORDER BY hand_id, seq",
            daybook::actions()
        ));
        let uuids: Vec<uuid::Uuid> = hands.iter().map(pokerkit::ID::inner).collect();
        let hrows = self.query(sql_h.as_str(), &[&uuids]).await?;
        let prows = self.query(sql_p.as_str(), &[&uuids]).await?;
        let arows = self.query(sql_a.as_str(), &[&uuids]).await?;
        let mut players: std::collections::HashMap<uuid::Uuid, Vec<Participant>> = std::collections::HashMap::new();
        let mut actions: std::collections::HashMap<uuid::Uuid, Vec<Play>> = std::collections::HashMap::new();
        for row in &prows {
            players.entry(row.get(0)).or_default().push(participant_from(row));
        }
        for row in &arows {
            actions.entry(row.get(0)).or_default().push(play_from(row));
        }
        Ok(hrows
            .iter()
            .map(|hr| {
                let hid: uuid::Uuid = hr.get(0);
                (hand_from(hr), players.remove(&hid).unwrap_or_default(), actions.remove(&hid).unwrap_or_default())
            })
            .collect())
    }

    async fn eval_hands_against(
        &self,
        user: ID<Member>,
        against: ID<Member>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ID<HandRecord>>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT h.id FROM {} h JOIN {} p1 ON p1.hand_id = h.id JOIN {} p2 ON p2.hand_id = h.id WHERE p1.user_id = $1 AND p2.user_id = $2 ORDER BY h.id DESC LIMIT $3 OFFSET $4",
            hands(), players(), players()
        ));
        self.query(sql.as_str(), &[&user.inner(), &against.inner(), &limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| ID::from(r.get::<_, uuid::Uuid>(0))).collect())
    }

    async fn eval_hands_by_stakes(
        &self,
        user: ID<Member>,
        stakes: i16,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ID<HandRecord>>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT h.id FROM {} h JOIN {} p ON p.hand_id = h.id JOIN {} r ON r.id = h.room_id WHERE p.user_id = $1 AND r.stakes = $2 ORDER BY h.id DESC LIMIT $3 OFFSET $4",
            hands(), players(), rooms()
        ));
        self.query(sql.as_str(), &[&user.inner(), &stakes, &limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| ID::from(r.get::<_, uuid::Uuid>(0))).collect())
    }

    async fn eval_pnl(&self, user: ID<Member>, limit: i64, offset: i64) -> Result<Vec<(Chips, Chips)>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT p.pnl, h.pot FROM {} h JOIN {} p ON p.hand_id = h.id WHERE p.user_id = $1 ORDER BY h.id DESC LIMIT $2 OFFSET $3",
            hands(), players()
        ));
        self.query(sql.as_str(), &[&user.inner(), &limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }

    async fn eval_pnl_against(
        &self,
        user: ID<Member>,
        against: ID<Member>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Chips, Chips)>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT p.pnl, h.pot FROM {} h JOIN {} p ON p.hand_id = h.id AND p.user_id = $1 JOIN {} p2 ON p2.hand_id = h.id AND p2.user_id = $2 ORDER BY h.id DESC LIMIT $3 OFFSET $4",
            hands(), players(), players()
        ));
        self.query(sql.as_str(), &[&user.inner(), &against.inner(), &limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }

    async fn eval_pnl_by_stakes(
        &self,
        user: ID<Member>,
        stakes: i16,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Chips, Chips)>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT p.pnl, h.pot FROM {} h JOIN {} p ON p.hand_id = h.id AND p.user_id = $1 JOIN {} r ON r.id = h.room_id WHERE r.stakes = $2 ORDER BY h.id DESC LIMIT $3 OFFSET $4",
            hands(), players(), rooms()
        ));
        self.query(sql.as_str(), &[&user.inner(), &stakes, &limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }

    async fn eval_pnl_human_hero(
        &self,
        _: &[uuid::Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Chips, Chips)>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT p.pnl, h.pot FROM {} h JOIN {} p ON p.hand_id = h.id WHERE p.user_id IS NULL ORDER BY h.id DESC LIMIT $1 OFFSET $2",
            hands(), players()
        ));
        self.query(sql.as_str(), &[&limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }

    async fn eval_pnl_human_against(
        &self,
        user: ID<Member>,
        _: &[uuid::Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Chips, Chips)>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT p.pnl, h.pot FROM {} h JOIN {} p ON p.hand_id = h.id AND p.user_id = $1 WHERE EXISTS (SELECT 1 FROM {} p2 WHERE p2.hand_id = h.id AND p2.user_id IS NULL) ORDER BY h.id DESC LIMIT $2 OFFSET $3",
            hands(), players(), players()
        ));
        self.query(sql.as_str(), &[&user.inner(), &limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }

    async fn eval_hands_human_hero(
        &self,
        _: &[uuid::Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ID<HandRecord>>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT h.id FROM {} h JOIN {} p ON p.hand_id = h.id WHERE p.user_id IS NULL ORDER BY h.id DESC LIMIT $1 OFFSET $2",
            hands(), players()
        ));
        self.query(sql.as_str(), &[&limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| ID::from(r.get::<_, uuid::Uuid>(0))).collect())
    }

    async fn eval_hands_human_against(
        &self,
        user: ID<Member>,
        _: &[uuid::Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ID<HandRecord>>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT h.id FROM {} h JOIN {} p ON p.hand_id = h.id AND p.user_id = $1 WHERE EXISTS (SELECT 1 FROM {} p2 WHERE p2.hand_id = h.id AND p2.user_id IS NULL) ORDER BY h.id DESC LIMIT $2 OFFSET $3",
            hands(), players(), players()
        ));
        self.query(sql.as_str(), &[&user.inner(), &limit, &offset])
            .await
            .map(|rows| rows.iter().map(|r| ID::from(r.get::<_, uuid::Uuid>(0))).collect())
    }

    async fn eval_count(&self, user: ID<Member>) -> Result<i64, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT COUNT(*) FROM {} WHERE user_id = $1", players()));
        self.query_one(sql.as_str(), &[&user.inner()])
            .await
            .map(|row| row.get(0))
    }

    async fn eval_count_against(&self, user: ID<Member>, against: ID<Member>) -> Result<i64, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT COUNT(*) FROM {} p JOIN {} p2 ON p2.hand_id = p.hand_id AND p2.user_id = $2 WHERE p.user_id = $1",
            players(), players()
        ));
        self.query_one(sql.as_str(), &[&user.inner(), &against.inner()])
            .await
            .map(|row| row.get(0))
    }

    async fn eval_count_by_stakes(&self, user: ID<Member>, stakes: i16) -> Result<i64, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT COUNT(*) FROM {} p JOIN {} h ON h.id = p.hand_id JOIN {} r ON r.id = h.room_id WHERE p.user_id = $1 AND r.stakes = $2",
            players(), hands(), rooms()
        ));
        self.query_one(sql.as_str(), &[&user.inner(), &stakes])
            .await
            .map(|row| row.get(0))
    }

    async fn eval_count_human_hero(&self, _: &[uuid::Uuid]) -> Result<i64, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT COUNT(*) FROM {} WHERE user_id IS NULL", players()));
        self.query_one(sql.as_str(), &[]).await.map(|row| row.get(0))
    }

    async fn eval_count_human_against(&self, user: ID<Member>, _: &[uuid::Uuid]) -> Result<i64, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!(
            "SELECT COUNT(*) FROM {} p WHERE p.user_id = $1 AND EXISTS (SELECT 1 FROM {} p2 WHERE p2.hand_id = p.hand_id AND p2.user_id IS NULL)",
            players(), players()
        ));
        self.query_one(sql.as_str(), &[&user.inner()])
            .await
            .map(|row| row.get(0))
    }

    async fn eval_policy(&self, past: i64, present: i16, choices: i64) -> Result<Vec<(i64, f32, f32)>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "SELECT edge, weight, payoff FROM {} WHERE past = $1 AND present = $2 AND choices = $3",
                blueprint()
            )
        });
        self.query(sql.as_str(), &[&past, &present, &choices])
            .await
            .map(|rows| rows.iter().map(|r| (r.get(0), r.get(1), r.get(2))).collect())
    }

    async fn eval_abstraction(&self, iso: i64) -> Result<Option<i16>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT abs FROM {} WHERE obs = $1", isomorphism()));
        self.query_opt(sql.as_str(), &[&iso])
            .await
            .map(|opt| opt.map(|row| row.get(0)))
    }

    async fn eval_abstractions(&self, isos: &[i64]) -> Result<Vec<(i64, i16)>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| format!("SELECT obs, abs FROM {} WHERE obs = ANY($1)", isomorphism()));
        self.query(sql.as_str(), &[&isos])
            .await
            .map(|rows| rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }

    async fn eval_chance_correction(
        &self,
        isos: &[i64],
        past: i64,
        choices: i64,
        observed_iso: i64,
    ) -> Result<Option<(f32, f32)>, PgErr> {
        static SQL: OnceLock<String> = OnceLock::<String>::new();
        let sql = SQL.get_or_init(|| {
            format!(
                "WITH deal_isos AS (\
                 SELECT unnest($1::bigint[]) AS obs\
             ), \
             deal_buckets AS (\
                 SELECT di.obs, i.abs \
                 FROM deal_isos di \
                 JOIN {iso} i ON i.obs = di.obs\
             ), \
             per_bucket AS (\
                 SELECT db.abs, \
                        SUM(b.weight * b.payoff) / NULLIF(SUM(b.weight), 0) AS baseline \
                 FROM (SELECT DISTINCT abs FROM deal_buckets) db \
                 JOIN {bp} b ON b.present = db.abs AND b.past = $2 AND b.choices = $3 \
                 GROUP BY db.abs\
             ), \
             observed AS (\
                 SELECT abs FROM {iso} WHERE obs = $4 LIMIT 1\
             ) \
             SELECT \
                 (SELECT (SUM(pb.baseline) / NULLIF(COUNT(*), 0))::float4 \
                  FROM deal_buckets db \
                  JOIN per_bucket pb ON pb.abs = db.abs), \
                 (SELECT baseline::float4 FROM per_bucket WHERE abs = (SELECT abs FROM observed))",
                iso = isomorphism(),
                bp = blueprint(),
            )
        });
        let row = self
            .query_opt(sql.as_str(), &[&isos, &past, &choices, &observed_iso])
            .await?;
        Ok(row.and_then(|r| {
            let avg: Option<f32> = r.get(0);
            let obs: Option<f32> = r.get(1);
            avg.zip(obs)
        }))
    }
}

fn hand_from(row: &tokio_postgres::Row) -> HandRecord {
    HandRecord::new(
        ID::from(row.get::<_, uuid::Uuid>(0)),
        ID::from(row.get::<_, uuid::Uuid>(1)),
        Board::from(deuce::Hand::from(row.get::<_, i64>(2) as u64)),
        row.get::<_, Chips>(3),
        row.get::<_, i16>(4) as Position,
    )
}

fn participant_from(row: &tokio_postgres::Row) -> Participant {
    Participant::with_visibility(
        ID::from(row.get::<_, uuid::Uuid>(0)),
        row.get::<_, Option<uuid::Uuid>>(1).map(ID::from),
        row.get::<_, i16>(2) as Position,
        Hole::from(deuce::Hand::from(row.get::<_, i64>(3) as u64)),
        row.get::<_, Chips>(4),
        Visibility::from(row.get::<_, i16>(5)),
        row.get::<_, Chips>(6),
    )
}

fn play_from(row: &tokio_postgres::Row) -> Play {
    Play::new(
        ID::from(row.get::<_, uuid::Uuid>(0)),
        row.get::<_, Epoch>(1),
        row.get::<_, Option<uuid::Uuid>>(2).map(ID::from),
        Action::from(row.get::<_, i32>(3) as u32),
        row.get::<_, Option<i32>>(4),
    )
}
