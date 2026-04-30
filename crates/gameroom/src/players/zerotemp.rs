//! Zero-temperature player that always takes the most likely action.
use crate::*;
use rand::prelude::*;
use rbp_gameplay::*;
use rbp_mccfr::*;
use rbp_nlhe::*;
use rbp_transport::Density;
use std::time::Duration;

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
                SubEdge::World(_) | SubEdge::Continuation(_) => None,
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
    fn argmax_blueprint(game: &Game, policy: Policy<NlheEdge>) -> Action {
        policy
            .support()
            .max_by(|a, b| {
                policy
                    .density(a)
                    .partial_cmp(&policy.density(b))
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
            Ok(Ok(policy)) => Self::argmax(&game, policy),
            _ => {
                let info = NlheInfo::from((recall.subgame(), abstraction, recall.choices()));
                Self::argmax_blueprint(&game, self.0.profile.averaged_distribution(&info))
            }
        }
    }
}
