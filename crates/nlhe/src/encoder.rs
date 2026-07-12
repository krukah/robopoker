use super::*;
use deuce::*;
use kicker::*;
use mccfr::*;
use pokerkit::*;
use std::collections::BTreeMap;
use subgame::*;

type NlheTree = Tree<NlheTurn, NlheEdge, NlheGame, NlheInfo>;

/// CfrEncoder that maps poker game states to information set identifiers.
///
/// Maintains a lookup table from suit-isomorphic hand representations
/// ([`Isomorphism`]) to strategic abstraction buckets ([`Abstraction`]).
/// This mapping is loaded from the database and represents the output of
/// the k-means clustering pipeline.
///
/// # Database Loading
///
/// With the `database` feature, implements `Hydrate` to load the
/// isomorphism→abstraction mapping from PostgreSQL.
#[derive(Default)]
pub struct NlheEncoder(BTreeMap<Isomorphism, Abstraction>);

impl NlheEncoder {
    /// Looks up the abstraction bucket for an observation.
    ///
    /// Internally converts to canonical isomorphism for lookup.
    /// Panics if the isomorphism is not in the lookup table.
    pub fn abstraction(&self, obs: &Observation) -> Abstraction {
        self.0
            .get(&Isomorphism::from(*obs))
            .copied()
            .expect("isomorphism not found in abstraction lookup")
    }
    /// Creates an info set for the root game state.
    pub fn root(&self, game: &NlheGame) -> NlheInfo {
        let subgame = Path::default();
        let present = self.abstraction(&game.sweat());
        let choices = game.as_ref().choices(0);
        NlheInfo::from((subgame, present, choices))
    }
}

impl mccfr::CfrEncoder for NlheEncoder {
    type T = NlheTurn;
    type E = NlheEdge;
    type G = NlheGame;
    type I = NlheInfo;

    fn seed(&self, root: &Self::G) -> Self::I {
        self.root(root)
    }

    fn info(&self, tree: &NlheTree, leaf: Leaf<Self::E, Self::G>) -> Self::I {
        NlheInfo::from((self, tree, leaf))
    }

    fn resume<P>(&self, past: P, game: &Self::G) -> Self::I
    where
        P: IntoIterator<Item = Self::E>,
    {
        let subgame = past.into_iter().map(Edge::from).collect::<Path>();
        let present = self.abstraction(&game.sweat());
        let choices = game.as_ref().choices(subgame.aggression());
        NlheInfo::from((subgame, present, choices))
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
        prefix: &Prefix<NlheTurn, NlheEdge>,
        game: &NlheGame,
        internal: NlheTurn,
        internal_bias: Continuation,
        external_bias: Continuation,
        profile: &P,
    ) -> Utility
    where
        P: RefProf<T = NlheTurn, E = NlheEdge, G = NlheGame, I = NlheInfo>,
    {
        let pos = Turn::from(internal).position();
        let mut game = Game::from(*game);
        let mut story: Story<NlheTurn, NlheEdge> = Story::from(prefix);
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
                    game = game.apply(game.reveal());
                }
                Turn::Choice(i) => {
                    let info = self.resume((&story).edges(), &NlheGame::from(game));
                    let dist = profile.averaged_distribution(&info);
                    let bias = if i == pos { internal_bias } else { external_bias };
                    let edge = Self::sample_biased(&dist, bias);
                    let action = game.actionize(Edge::from(edge));
                    story.push(Descent(NlheTurn::from(i), edge));
                    game = game.apply(game.snap(action));
                }
            }
        }
    }
    /// Sample an edge from a biased distribution.
    ///
    /// The target action type probability is multiplied by the runtime-configured
    /// `FrontierHyperParams::get().bias()` (default 5.0), then renormalized.
    fn sample_biased(dist: &Policy<NlheEdge>, bias: Continuation) -> NlheEdge {
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
impl<const W: usize> WorldRestrict<W> for NlheEncoder {
    /// Sample an opponent hole consistent with the tracked belief and the
    /// requested world. If rejection sampling exhausts (rare — happens when
    /// the belief has been narrowed to a set the random sampler can't hit
    /// in `MAX_REJECTIONS` draws), fall back to an unconstrained random
    /// hole rather than panicking. Losing world consistency for one hand
    /// is far better than crashing a 10-hour benchmark task.
    fn restrict(
        &self,
        external: Self::T,
        world: World,
        belief: &Belief<Secret<Self>, W>,
        observed: &Self::G,
    ) -> Self::G {
        const MAX_REJECTIONS: usize = 10_000;
        let position = Turn::from(external).position();
        let baseline = Game::from(*observed);
        let available = Hand::or(Hand::from(baseline.deck()), Hand::from(baseline.seats()[position].cards()));
        std::iter::repeat_with(|| Deck::from(available).hole())
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
                    NlheGame::from(baseline.deal(position, Deck::from(available).hole()))
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
            let obs = Isomorphism::from(row.get::<_, i64>(0));
            let abs = Abstraction::from(row.get::<_, i16>(1));
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
