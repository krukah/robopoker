use super::*;
use rbp_cards::*;
use rbp_gameplay::*;
use rbp_mccfr::*;
use std::collections::BTreeMap;

type NlheTree = Tree<NlheTurn, NlheEdge, NlheGame, NlheInfo>;

/// Encoder that maps poker game states to information set identifiers.
///
/// Maintains a lookup table from suit-isomorphic hand representations
/// ([`Isomorphism`]) to strategic abstraction buckets ([`Abstraction`]).
/// This mapping is loaded from the database and represents the output of
/// the k-means clustering pipeline.
///
/// # Database Loading
///
/// With the `database` feature, implements [`Hydrate`] to load the
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

impl rbp_mccfr::Encoder for NlheEncoder {
    type T = NlheTurn;
    type E = NlheEdge;
    type G = NlheGame;
    type I = NlheInfo;
    fn seed(&self, root: &Self::G) -> Self::I {
        self.root(root)
    }
    fn info(&self, tree: &NlheTree, leaf: Branch<Self::E, Self::G>) -> Self::I {
        NlheInfo::from((self, tree, leaf))
    }
    fn resume(&self, past: &[Self::E], game: &Self::G) -> Self::I {
        // THERE MAY BE TRUNCATION HERE
        // BUT i think it's okay? SUBGAME_DEPTH ?
        let subgame = past.iter().map(|e| Edge::from(*e)).collect::<Path>();
        let present = self.abstraction(&game.sweat());
        let choices = game.as_ref().choices(subgame.aggression());
        NlheInfo::from((subgame, present, choices))
    }
}

#[cfg(feature = "database")]
#[async_trait::async_trait]
impl rbp_pg::Hydrate for NlheEncoder {
    async fn hydrate(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        log::info!("{:<32}{:<32}", "loading isomorphism", "from database");
        let sql = const_format::concatcp!("SELECT obs, abs FROM ", rbp_pg::ISOMORPHISM);
        let lookup = client
            .query(sql, &[])
            .await
            .expect("isomorphism query")
            .into_iter()
            .map(|row| (row.get::<_, i64>(0), row.get::<_, i16>(1)))
            .map(|(obs, abs)| (Isomorphism::from(obs), Abstraction::from(abs)))
            .collect::<BTreeMap<Isomorphism, Abstraction>>();
        Self(lookup)
    }
}
