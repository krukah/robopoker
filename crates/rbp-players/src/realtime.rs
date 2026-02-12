//! Subgame-solving player that refines blueprint at decision time.
use rbp_gameplay::*;
use rbp_gameroom::*;
use rbp_mccfr::*;
use rbp_nlhe::*;
use rbp_transport::Density;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;

/// Compute player using real-time subgame solving.
///
/// Refines blueprint strategies at decision time using safe subgame solving.
/// Slower than DatabasePlayer but produces stronger play by adapting to the
/// specific game state reached.
pub struct RealTimePlayer(&'static Flagship);

impl RealTimePlayer {
    /// Creates a new subgame player from static blueprint reference.
    pub fn new(blueprint: &'static Flagship) -> Self {
        Self(blueprint)
    }
    /// Creates a subgame player by loading from database and leaking.
    #[cfg(feature = "database")]
    pub async fn from_database(client: std::sync::Arc<tokio_postgres::Client>) -> Self {
        use rbp_pg::Hydrate;
        Self(Box::leak(Box::new(Flagship::hydrate(client).await)))
    }
    /// Samples an action from subgame policy using weighted random selection.
    fn sample(game: &Game, policy: Policy<SubEdge<NlheEdge>>) -> Action {
        let edges = policy
            .support()
            .filter_map(|e| match e {
                SubEdge::Inner(e) => Some(e),
                SubEdge::World(_) => None,
            })
            .collect::<Vec<_>>();
        let weights = edges
            .iter()
            .map(|e| policy.density(&SubEdge::Inner(*e)))
            .collect::<Vec<_>>();
        WeightedIndex::new(&weights)
            .ok()
            .map(|dist| edges[dist.sample(&mut rand::rng())])
            .map(|edge| game.actionize(Edge::from(edge)))
            .unwrap_or_else(|| game.legal().choose(&mut rand::rng()).copied().unwrap())
    }
}

#[async_trait::async_trait]
impl Player for RealTimePlayer {
    async fn notify(&mut self, _: &Event) {}
    async fn decide(&mut self, recall: &Partial) -> Action {
        let game = recall.head();
        let observation = recall.seen();
        let abstraction = self.0.encoder().abstraction(&observation);
        let info = SubInfo::Info(NlheInfo::from((recall, abstraction)));
        let solver = self.0.subgame(recall);
        let policy = solver.solve().profile().averaged_distribution(&info);
        Self::sample(&game, policy)
    }
}
