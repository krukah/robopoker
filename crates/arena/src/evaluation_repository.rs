use bouncer::Member;
use daybook::*;
use deuce::*;
use kicker::*;
use nlhe::NlheSecret;
use parlor::records::{Hand as HandRecord, Participant, Play};
use pokerkit::*;

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
    async fn eval_policy(
        &self,
        past: Subgame,
        present: NlheSecret,
        choices: Path,
        context: Field,
    ) -> Result<Vec<(Edge, f32, f32)>, PgErr>;
    async fn eval_abstraction(&self, iso: Isomorphism) -> Result<Option<Abstraction>, PgErr>;
    async fn eval_chance_correction(
        &self,
        isos: &[Isomorphism],
        past: Subgame,
        choices: Path,
        observed_iso: Isomorphism,
    ) -> Result<Option<(f32, f32)>, PgErr>;
}
