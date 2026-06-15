use arena::*;
use bouncer::Member;
use deuce::*;
use kicker::*;
use parlor::records::{Hand as HandRecord, Participant, Visibility};
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
            .map(|v| v.uuid())
            .chain(std::iter::once(pokerkit::slumbot_opponent_uuid()))
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
        let mut summary = summarize_pnl(&rows);
        summary.population = population as usize;
        Ok(summary)
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
        let hands = match (hero_human, against_human, against, stakes) {
            (true, _, _, _) => self.0.eval_hands_human_hero(&Self::bot_uuids(), limit, offset).await?,
            (_, true, _, _) => {
                self.0
                    .eval_hands_human_against(uid, &Self::bot_uuids(), limit, offset)
                    .await?
            }
            (_, _, Some(opp), _) => self.0.eval_hands_against(uid, ID::from(opp), limit, offset).await?,
            (_, _, _, Some(s)) => self.0.eval_hands_by_stakes(uid, s, limit, offset).await?,
            _ => self.0.eval_hands(uid, limit, offset).await?,
        };
        let bundles = self.0.eval_bundles(&hands).await?;
        let aivat = Aivat::new(self.0.clone());
        let mut recaps = Vec::with_capacity(bundles.len());
        let mut results = Vec::with_capacity(bundles.len());
        for (h, p, a) in &bundles {
            if let Some(s) = seat_of(p, uid)
                && let Ok(recap) = replay(h, p, a, s)
                && let Ok(result) = aivat.evaluate(h, p, a, s, &recap).await
            {
                recaps.push(recap);
                results.push(result);
            }
        }
        let summary = summarize(&recaps);
        Ok(Aivat::summarize(&summary, &results))
    }

    pub async fn hand_recap(&self, id: uuid::Uuid) -> anyhow::Result<ApiRecap> {
        let (hand, parts, plays) = self.0.eval_bundle(ID::<HandRecord>::from(id)).await?;
        let board = Vec::<Card>::from(deuce::Hand::from(hand.board()))
            .iter()
            .map(|c| format!("{c}"))
            .collect::<Vec<_>>()
            .join(" ");
        let obs = Observation::from((Hand::from(parts[0].hole()), Hand::from(hand.board())));
        let witness = plays.iter().filter(|p| !p.action().is_blind()).try_fold(
            Witness::initial_with(Turn::Choice(0), Arrangement::from(obs), stacks(&parts)?, hand.dealer()),
            |r, p| r.try_push(p.action()),
        )?;
        let actions = witness
            .plays()
            .into_iter()
            .enumerate()
            .map(|(seq, (_, action, street))| ApiPlay {
                seq: seq as i16,
                action: action.to_string(),
                street: street.to_string(),
            })
            .collect();
        Ok(ApiRecap {
            pot: hand.pot(),
            board,
            dealer: hand.dealer(),
            players: parts
                .iter()
                .map(|p| ApiPlayer {
                    seat: p.seat(),
                    stack: p.stack(),
                    hole: match p.visibility() {
                        Visibility::Showed => Some(format!("{}", p.hole())),
                        _ => None,
                    },
                    won: p.pnl(),
                })
                .collect(),
            actions,
        })
    }
}

fn seat_of(parts: &[Participant], uid: ID<Member>) -> Option<Position> {
    parts
        .iter()
        .find(|p| p.user() == Some(uid))
        .map(parlor::Participant::seat)
}
