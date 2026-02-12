//! Zero-temperature player that always takes the most likely action.
use rbp_gameplay::*;
use crate::*;
use rbp_mccfr::*;
use rbp_nlhe::*;
use rbp_transport::Density;
use rand::prelude::*;

/// Compute player using subgame solving with deterministic action selection.
///
/// Like SubgamePlayer but always selects the highest-probability action
/// rather than sampling. Zero temperature in the softmax sense.
pub struct ZeroTempPlayer(&'static Flagship);

impl ZeroTempPlayer {
    /// Creates a new zero-temperature player from static blueprint reference.
    pub fn new(blueprint: &'static Flagship) -> Self {
        Self(blueprint)
    }
    /// Creates a zero-temperature player by loading from database and leaking.
    #[cfg(feature = "database")]
    pub async fn from_database(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        use rbp_database::Hydrate;
        Self(Box::leak(Box::new(Flagship::hydrate(client).await)))
    }
    /// Selects the highest-probability action from subgame policy (argmax).
    fn argmax(game: &Game, policy: Policy<SubEdge<NlheEdge>>) -> Action {
        policy
            .support()
            .filter_map(|e| match e {
                SubEdge::Inner(e) => Some(e),
                SubEdge::World(_) => None,
            })
            .max_by(|a, b| {
                policy
                    .density(&SubEdge::Inner(*a))
                    .partial_cmp(&policy.density(&SubEdge::Inner(*b)))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|edge| game.actionize(Edge::from(edge)))
            .unwrap_or_else(|| game.legal().choose(&mut rand::rng()).copied().unwrap())
    }
}

#[async_trait::async_trait]
impl Player for ZeroTempPlayer {
    async fn notify(&mut self, _: &Event) {}
    async fn decide(&mut self, recall: &Partial) -> Action {
        let game = recall.head();
        let observation = recall.seen();
        let abstraction = self.0.encoder().abstraction(&observation);
        let info = SubInfo::Info(NlheInfo::from((recall, abstraction)));
        let solver = self.0.subgame(recall);
        let policy = solver.solve().profile().averaged_distribution(&info);
        Self::argmax(&game, policy)
    }
}
