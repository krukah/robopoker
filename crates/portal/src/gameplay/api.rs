use arena::*;
use bouncer::Member;
use kicker::*;
use parlor::records::{Hand as HandRecord, Participant, Play, Visibility};
use pokerkit::*;
use std::sync::Arc;
use tokio_postgres::Client;

pub struct GameplayAPI(Arc<Client>);

impl GameplayAPI {
    pub fn new(client: Arc<Client>) -> Self {
        Self(client)
    }

    fn bot_uuids() -> Vec<uuid::Uuid> {
        pokerkit::Variant::all()
            .iter()
            .copied()
            .map(pokerkit::Variant::uuid)
            .chain(
                pokerkit::Adversary::all()
                    .iter()
                    .copied()
                    .map(pokerkit::Adversary::uuid),
            )
            .collect()
    }

    pub async fn summary(
        &self,
        user: uuid::Uuid,
        limit: i64,
        offset: i64,
        against: Option<uuid::Uuid>,
        stakes: Option<i16>,
        hero_human: bool,
        against_human: bool,
    ) -> anyhow::Result<Summary> {
        let uid = ID::<Member>::from(user);
        let bots = Self::bot_uuids();
        let (rows, population) = match (hero_human, against_human, against, stakes) {
            (true, _, _, _) => {
                (self.0.eval_pnl_human_hero(&bots, limit, offset).await?, self.0.eval_count_human_hero(&bots).await?)
            }
            (_, true, _, _) => (
                self.0.eval_pnl_human_against(uid, &bots, limit, offset).await?,
                self.0.eval_count_human_against(uid, &bots).await?,
            ),
            (_, _, Some(opp), _) => (
                self.0.eval_pnl_against(uid, ID::from(opp), limit, offset).await?,
                self.0.eval_count_against(uid, ID::from(opp)).await?,
            ),
            (_, _, _, Some(s)) => {
                (self.0.eval_pnl_by_stakes(uid, s, limit, offset).await?, self.0.eval_count_by_stakes(uid, s).await?)
            }
            _ => (self.0.eval_pnl(uid, limit, offset).await?, self.0.eval_count(uid).await?),
        };
        Ok(Summary {
            population: population as usize,
            ..Metrics::summarize_pnl(&rows)
        })
    }

    /// The hands a matchup filter selects, newest first.
    ///
    /// AIVAT and the hand list ask exactly this question, and asked it in
    /// two places until the second one needed asking.
    async fn hands(
        &self,
        uid: ID<Member>,
        limit: i64,
        offset: i64,
        against: Option<uuid::Uuid>,
        stakes: Option<i16>,
        hero_human: bool,
        against_human: bool,
    ) -> anyhow::Result<Vec<ID<HandRecord>>> {
        Ok(match (hero_human, against_human, against, stakes) {
            (true, _, _, _) => self.0.eval_hands_human_hero(&Self::bot_uuids(), limit, offset).await?,
            (_, true, _, _) => {
                self.0
                    .eval_hands_human_against(uid, &Self::bot_uuids(), limit, offset)
                    .await?
            }
            (_, _, Some(opp), _) => self.0.eval_hands_against(uid, ID::from(opp), limit, offset).await?,
            (_, _, _, Some(s)) => self.0.eval_hands_by_stakes(uid, s, limit, offset).await?,
            _ => self.0.eval_hands(uid, limit, offset).await?,
        })
    }

    /// A page of stored hands, each one reopenable.
    ///
    /// The list *is* the detail: a recap carries its own line, so browsing
    /// hands and studying one are not two round trips. Every hand is read
    /// from the reviewing user's seat, because that is whose hand it is.
    pub async fn recaps(
        &self,
        user: uuid::Uuid,
        limit: i64,
        offset: i64,
        against: Option<uuid::Uuid>,
        stakes: Option<i16>,
        hero_human: bool,
        against_human: bool,
    ) -> anyhow::Result<Vec<ApiRecap>> {
        let uid = ID::<Member>::from(user);
        let hands = self
            .hands(uid, limit, offset, against, stakes, hero_human, against_human)
            .await?;
        Ok(self
            .0
            .eval_bundles(&hands)
            .await?
            .iter()
            .filter_map(|(hand, parts, plays)| {
                GameplayAPI::recap(hand, parts, plays, GameplayAPI::seat_of(parts, uid).unwrap_or_default()).ok()
            })
            .collect())
    }

    pub async fn aivat(
        &self,
        user: uuid::Uuid,
        limit: i64,
        offset: i64,
        against: Option<uuid::Uuid>,
        stakes: Option<i16>,
        hero_human: bool,
        against_human: bool,
    ) -> anyhow::Result<AivatDelta> {
        let uid = ID::<Member>::from(user);
        let hands = self
            .hands(uid, limit, offset, against, stakes, hero_human, against_human)
            .await?;
        let bundles = self.0.eval_bundles(&hands).await?;
        let aivat = Aivat::new(self.0.clone());
        let mut recaps = Vec::with_capacity(bundles.len());
        let mut results = Vec::with_capacity(bundles.len());
        for (h, p, a) in &bundles {
            if let Some(s) = GameplayAPI::seat_of(p, uid)
                && let Ok(recap) = Replay::replay(h, p, a, s)
                && let Ok(result) = aivat.evaluate(h, p, a, s, &recap).await
            {
                recaps.push(recap);
                results.push(result);
            }
        }
        let summary = Metrics::summarize(&recaps);
        Ok(Aivat::summarize(&summary, &results))
    }

    pub async fn hand_recap(&self, id: uuid::Uuid) -> anyhow::Result<ApiRecap> {
        let (hand, parts, plays) = self.0.eval_bundle(ID::<HandRecord>::from(id)).await?;
        GameplayAPI::recap(&hand, &parts, &plays, 0)
    }
}

impl GameplayAPI {
    /// One stored hand, read from `seat`.
    ///
    /// The reconstruction is [`arena::replay`]'s, not a second one written
    /// here. This used to rebuild its own witness from `parts[0]` and a board
    /// built out of order, which is two ways of saying the same thing and one
    /// of them subtly wrong.
    fn recap(hand: &HandRecord, parts: &[Participant], plays: &[Play], seat: Position) -> anyhow::Result<ApiRecap> {
        Ok(ApiRecap {
            id: hand.id().inner(),
            pot: hand.pot(),
            dealer: hand.dealer(),
            recall: Replay::replay(hand, parts, plays, seat)?.witness().clone(),
            // Visibility is the whole rule: a seat that showed is known, a seat that mucked or was
            // never asked is not, and the absence says so without a second field to contradict it.
            reveals: (0..N)
                .map(|seat| {
                    parts
                        .iter()
                        .find(|p| p.seat() == seat)
                        .filter(|p| p.visibility() == Visibility::Showed)
                        .map(Participant::hole)
                })
                .collect(),
            players: parts
                .iter()
                .map(|p| ApiPlayer {
                    seat: p.seat(),
                    stack: p.stack(),
                    won: p.pnl(),
                })
                .collect(),
        })
    }

    fn seat_of(parts: &[Participant], uid: ID<Member>) -> Option<Position> {
        parts
            .iter()
            .find(|p| p.user() == Some(uid))
            .map(parlor::Participant::seat)
    }
}
