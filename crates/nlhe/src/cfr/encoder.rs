use super::*;
use deuce::*;
use kicker::*;
use mccfr::*;
use pokerkit::*;
use std::collections::BTreeMap;
use subgame::*;

/// Maps poker game states to info-set identifiers, via a lookup table from
/// suit-isomorphic hands ([`Isomorphism`]) to strategic abstraction buckets
/// ([`Abstraction`]) — the output of the k-means clustering pipeline, hydrated
/// from PostgreSQL under the `database` feature.
#[derive(Default)]
pub struct NlheEncoder(BTreeMap<Isomorphism, Abstraction>);

impl NlheEncoder {
    /// Abstraction bucket for an observation, via its canonical isomorphism.
    /// Panics if the isomorphism is not in the lookup table.
    pub fn abstraction(&self, obs: &Observation) -> Abstraction {
        self.0
            .get(&Isomorphism::from(*obs))
            .copied()
            .expect("isomorphism not found in abstraction lookup")
    }
}

impl Retrace for NlheEncoder {}

impl mccfr::CfrSpec for NlheEncoder {
    type F = NlheInfo;
}

impl mccfr::Encoding for NlheEncoder {
    fn public<P>(&self, game: &GameOf<Self>, past: P) -> PublicOf<Self>
    where
        P: IntoIterator<Item = EdgeOf<Self>>,
    {
        NlhePublic::at(game, past)
    }

    fn secret(&self, game: &GameOf<Self>) -> SecretOf<Self> {
        NlheSecret::from(self.abstraction(&game.sweat()))
    }
}

impl NlheEncoder {
    /// Single Monte Carlo rollout from a frontier game state under biased strategies.
    ///
    /// The `prefix` is the immutable context predating the rollout — typically
    /// the subgame solver's construction prefix. The rollout's own growing
    /// path is a [`Story`], distinct by type so it can't be fed back
    /// in as a prefix by mistake.
    pub(crate) fn biased_rollout<P>(
        &self,
        prefix: &Prefix<NlheGame>,
        game: &NlheGame,
        internal: NlheTurn,
        internal_bias: Continuation,
        external_bias: Continuation,
        profile: &P,
    ) -> Utility
    where
        P: Profile<F = NlheInfo>,
    {
        let pos = Turn::from(internal).position();
        let mut game = Game::from(*game);
        let mut story: Story<NlheGame> = Story::from(prefix);
        loop {
            match game.turn() {
                Turn::Terminal => {
                    return game
                        .settlements()
                        .get(pos)
                        .map(|s| s.won() as Utility)
                        .expect("valid position");
                }
                Turn::Chance => {
                    story.push(Descent(NlheTurn::chance(), NlheEdge::from(Edge::Draw)));
                    game = game.apply(game.reveal(&mut rand::rng()));
                }
                Turn::Choice(i) => {
                    let info = self.encode(&NlheGame::from(game), story.iter().rev().map(|d| d.1));
                    let dist = profile.averaged_strategy(&info);
                    let bias = if i == pos { internal_bias } else { external_bias };
                    let edge = Self::sample_biased(&dist, bias);
                    let action = game.actionize(Edge::from(edge));
                    story.push(Descent(NlheTurn::from(i), edge));
                    game = game.apply(game.snap(action));
                }
            }
        }
    }
    /// Sample an edge, with the target action type's probability multiplied by
    /// `FrontierHyperParams::get().bias()` (default 5.0), then renormalized.
    fn sample_biased(dist: &Over<NlheEdge, Policy>, bias: Continuation) -> NlheEdge {
        let bias_mult = FrontierHyperParams::get().bias();
        let biased = dist
            .iter()
            .map(|&(e, p)| {
                let inner = Edge::from(e);
                let mult = match bias.index() {
                    3 if inner.is_aggro() => bias_mult,
                    1 if inner.is_folded() => bias_mult,
                    2 if !inner.is_folded() && !inner.is_aggro() => bias_mult,
                    _ => 1.0,
                };
                (e, p * mult)
            })
            .collect::<Vec<_>>();
        let total = biased.iter().map(|(_, p)| *p).sum::<Probability>();
        let threshold = rand::random::<Probability>() * total;
        biased
            .iter()
            .scan(0.0, |acc, &(e, p)| {
                *acc += p;
                Some((e, *acc))
            })
            .find(|&(_, acc)| threshold < acc)
            .map_or(biased.last().expect("non-empty distribution").0, |(e, _)| e)
    }
}
/// Frontier payoffs from biased rollouts: each `(k, j)` cell averages rollouts where the two
/// players follow continuations `k` and `j` of the blueprint.
impl Frontier<{ pokerkit::FRONTIER_LEAVES }> for NlheEncoder {
    fn payoffs<P>(
        &self,
        blueprint: &P,
        prefix: &Prefix<NlheGame>,
        game: &NlheGame,
        internal: NlheTurn,
    ) -> Payoffs<{ pokerkit::FRONTIER_LEAVES }>
    where
        P: Profile<F = NlheInfo>,
    {
        let rollouts = FrontierHyperParams::get().rollouts();
        Payoffs::tabulate(|k, j| {
            (0..rollouts)
                .map(|_| self.biased_rollout(prefix, game, internal, k, j, blueprint))
                .sum::<Utility>()
                / rollouts as Utility
        })
    }
}

impl<const W: usize> WorldRestrict<W> for NlheEncoder {
    /// Sample an opponent hole consistent with the tracked belief and the requested world. If
    /// rejection sampling exhausts (rare — happens when the belief has been narrowed to a set
    /// the random sampler can't hit in `MAX_REJECTIONS` draws), fall back to an unconstrained
    /// random hole rather than panicking. Losing world consistency for one hand is far better
    /// than crashing a 10-hour benchmark task.
    fn restrict(
        &self,
        external: TurnOf<Self>,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &GameOf<Self>,
    ) -> GameOf<Self> {
        const MAX_REJECTIONS: usize = 10_000;
        let position = Turn::from(external).position();
        let baseline = Game::from(*observed);
        let available = Hand::or(Hand::from(baseline.deck()), Hand::from(baseline.seats()[position].cards()));
        std::iter::repeat_with(|| Deck::from(available).hole(&mut rand::rng()))
            .take(MAX_REJECTIONS)
            .map(|hole| baseline.deal(position, hole))
            .map(NlheGame::from)
            .map(|game| (game, game.sweat_at(position)))
            .map(|(game, obs)| (game, self.abstraction(&obs)))
            .map(|(game, abs)| (game, NlheSecret::from(abs)))
            .find(|(_, secret)| belief.remember(secret, world))
            .map_or_else(
                || {
                    tracing::warn!(
                        world = world.index(),
                        max_rejections = MAX_REJECTIONS,
                        "rejection sampling exhausted; falling back to unconstrained hole",
                    );
                    NlheGame::from(baseline.deal(position, Deck::from(available).hole(&mut rand::rng())))
                },
                |(game, _)| game,
            )
    }
}

#[cfg(feature = "server")]
#[async_trait::async_trait]
impl daybook::Hydrate for NlheEncoder {
    /// Streams rows from the database one at a time, avoiding the peak
    /// memory spike of buffering all 138M rows as `Vec<Row>`.
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        use futures::StreamExt;
        tracing::info!("{:<32}{:<32}", "loading isomorphism", "from database");
        let sql = format!("SELECT obs, abs FROM {}", daybook::isomorphism());
        let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        let stream = client.query_raw(&sql, params).await.expect("isomorphism query");
        futures::pin_mut!(stream);
        let mut lookup = BTreeMap::new();
        let mut count = 0usize;
        while let Some(row) = stream.next().await {
            let row = row.expect("isomorphism row");
            let obs = row.get::<_, Isomorphism>(0);
            let abs = row.get::<_, Abstraction>(1);
            lookup.insert(obs, abs);
            count += 1;
            if count.is_multiple_of(10_000_000) {
                tracing::info!("{:<32}{:<32}", format!("{count:>16} isomorphisms"), "from database");
            }
        }
        tracing::info!("{:<32}{:<32}", format!("{count:>16} isomorphisms"), "from database");
        Self(lookup)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl NlheEncoder {
        /// Minimal encoder: `encode` only looks up the game's own observation,
        /// so one entry suffices.
        fn minimal(game: &NlheGame) -> Self {
            Self(std::iter::once((Isomorphism::from(game.sweat()), Abstraction::default())).collect())
        }
    }

    /// The regression that killed every `world` variant of the 2026-09-07 slumbot run: a solver
    /// tree that crosses chance nodes fed `encode` a multi-street edge list, which raw
    #[test]
    fn encode_survives_a_path_longer_than_subgame_capacity() {
        let street = || {
            [Edge::Check, Edge::Call]
                .into_iter()
                .chain(std::iter::once(Edge::Draw))
                .map(NlheEdge::from)
        };
        let past = std::iter::repeat_with(street).take(8).flatten().collect::<Vec<_>>();
        assert!(past.len() > Subgame::CAPACITY, "fixture must exceed capacity");
        let game = NlheGame::root();
        let info = NlheEncoder::minimal(&game).encode(&game, past.into_iter().rev());
        assert_eq!(Vec::<Edge>::from(info.subgame()).len(), 0, "trailing Draw resets the street");
    }

    /// The emitted key holds one street. Invariant, not a regression: it
    /// held before this change too, via `NlheInfo::from`'s trim.
    #[test]
    fn encode_resets_the_subgame_at_every_chance_edge() {
        let past = [Edge::Check, Edge::Call, Edge::Draw, Edge::Check]
            .into_iter()
            .map(NlheEdge::from)
            .collect::<Vec<_>>();
        let game = NlheGame::root();
        let info = NlheEncoder::minimal(&game).encode(&game, past.into_iter().rev());
        assert_eq!(Vec::<Edge>::from(info.subgame()), vec![Edge::Check], "only the post-Draw street survives");
    }

    /// Within one street the key is the full action sequence, unchanged.
    #[test]
    fn encode_keeps_a_single_street_intact() {
        let past = [Edge::Check, Edge::Call]
            .into_iter()
            .map(NlheEdge::from)
            .collect::<Vec<_>>();
        let game = NlheGame::root();
        let info = NlheEncoder::minimal(&game).encode(&game, past.into_iter().rev());
        assert_eq!(Vec::<Edge>::from(info.subgame()), vec![Edge::Check, Edge::Call]);
    }
}
