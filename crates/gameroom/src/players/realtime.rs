//! Subgame-solving player that refines blueprint at decision time.
use crate::*;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use rbp_gameplay::*;
use rbp_mccfr::*;
use rbp_nlhe::*;
use rbp_transport::Density;
use std::time::Duration;

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
        use rbp_database::Hydrate;
        Self(Box::leak(Box::new(Flagship::hydrate(client).await)))
    }
    /// Samples an action from subgame policy using weighted random selection.
    fn sample(game: &Game, policy: Policy<SubEdge<NlheEdge>>) -> Action {
        let edges = policy
            .support()
            .filter_map(|e| match e {
                SubEdge::Inner(e) => Some(e),
                SubEdge::World(_) | SubEdge::Continuation(_) => None,
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
    fn sample_blueprint(game: &Game, policy: Policy<NlheEdge>) -> Action {
        let edges = policy.support().collect::<Vec<_>>();
        let weights = edges
            .iter()
            .map(|edge| policy.density(edge))
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
        let Some(abstraction) = self.0.encoder().try_abstraction(&observation) else {
            return game.legal().choose(&mut rand::rng()).copied().unwrap();
        };
        if has_offtree_actions(recall) {
            log::debug!("off-tree action sequence detected; DLS will use canonicalized edges");
        }
        let blueprint = self.0;
        let recall_for_solve = recall.clone();
        let info = SubInfo::Info(NlheInfo::from((&recall_for_solve, abstraction)));
        let solve = tokio::task::spawn_blocking(move || {
            let solver = blueprint.depth_limited_subgame(&recall_for_solve);
            solver.solve().profile().averaged_distribution(&info)
        });
        let timeout = Duration::from_millis(DlsOptions::default().max_solve_ms);
        match tokio::time::timeout(timeout, solve).await {
            Ok(Ok(policy)) => Self::sample(&game, policy),
            _ => {
                let info = NlheInfo::from((recall.subgame(), abstraction, recall.choices()));
                Self::sample_blueprint(&game, self.0.profile.averaged_distribution(&info))
            }
        }
    }
}
