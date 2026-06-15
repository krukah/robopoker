use cowboys::*;
use holdem::*;
use ledger::*;
use mccfr::Solver;
use parlor::Solved;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;
use tokio_postgres::Client;

/// Default per-decision deadline for runtime subgame solvers in
/// analysis mode. Picked to feel responsive while still letting the
/// solver converge on common spots; clients may parameterize it later
/// (e.g. a "give it longer" knob in the panel).
const DEFAULT_SOLVE_DEADLINE_MS: u64 = 1000;

/// Toggle for the (Witness, Kind) → ApiSolved cache. False initially —
/// the user wants to observe variance across re-runs of the stochastic
/// solvers. Flip to `true` to start memoizing identical requests.
const CACHE_ENABLED: bool = false;

fn cache() -> &'static Mutex<HashMap<(Witness, Kind), ApiSolved>> {
    static CACHE: OnceLock<Mutex<HashMap<(Witness, Kind), ApiSolved>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Memoization wrapper for expensive solves. With `CACHE_ENABLED=false`
/// it's a no-op pass-through — every call re-runs `compute`. When the
/// flag flips, identical `(Witness, Kind)` keys hit the cache directly
/// without re-running the solver.
async fn cache_or<F, Fut>(key: (Witness, Kind), compute: F) -> ApiSolved
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ApiSolved>,
{
    if !CACHE_ENABLED {
        return compute().await;
    }
    if let Some(hit) = cache().lock().unwrap().get(&key).cloned() {
        return hit;
    }
    let result = compute().await;
    cache().lock().unwrap().insert(key, result.clone());
    result
}

fn api_strategy_from(strategy: Strategy, recall: &Witness) -> ApiStrategy {
    ApiStrategy {
        history: strategy.info().subgame(),
        present: Abstraction::from(strategy.info().bucket()),
        choices: strategy.info().choices(),
        spr: recall.head().geometry().tag(),
        accumulated: strategy.accumulated().clone(),
        visits: strategy.visits().clone(),
        payoff: strategy.payoff(),
    }
}

fn street_name(s: i16) -> &'static str {
    match s {
        0 => "preflop",
        1 => "flop",
        2 => "turn",
        3 => "river",
        _ => "unknown",
    }
}

/// Runs the solver for `kind` synchronously and assembles the
/// `ApiSolved` envelope. Pulled out so [`StrategyAPI::solve`] can run
/// it in a blocking thread without straddling the `Nlhe` borrow
/// across an `await`.
fn run_solve(blueprint: &'static Flagship, recall: &Witness, kind: Kind) -> ApiSolved {
    let info = NlheInfo::from((recall, blueprint.encoder().abstraction(&recall.seen())));
    let deadline = Duration::from_millis(DEFAULT_SOLVE_DEADLINE_MS);
    let solved = match kind {
        Kind::Depth => Solved::run(blueprint.adapt_leaf(recall), info, deadline),
        Kind::World => Solved::run(blueprint.adapt_safe(recall), info, deadline),
        Kind::Full => Solved::run(blueprint.adapt_full(recall), info, deadline),
        Kind::Blueprint => unreachable!("Kind::Blueprint goes through the lookup path, not solve"),
    };
    let policy = ApiStrategy {
        history: info.subgame(),
        present: Abstraction::from(info.bucket()),
        choices: info.choices(),
        spr: recall.head().geometry().tag(),
        accumulated: solved.policy().clone(),
        visits: solved.visits().clone(),
        payoff: 0.0,
    };
    ApiSolved {
        kind,
        policy,
        iterations: solved.iterations() as u32,
        elapsed_ms: solved.elapsed().as_millis() as u32,
    }
}

pub struct StrategyAPI {
    client: Arc<Client>,
    blueprint: Option<&'static holdem::Flagship>,
}

impl StrategyAPI {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            blueprint: None,
        }
    }

    pub fn with_blueprint(mut self, blueprint: Option<&'static holdem::Flagship>) -> Self {
        self.blueprint = blueprint;
        self
    }

    pub async fn policy(&self, recall: Witness) -> anyhow::Result<Option<ApiStrategy>> {
        let recall = recall.validate()?;
        Ok(holdem::lookup(&self.client, &recall)
            .await
            .map(|s| api_strategy_from(s, &recall)))
    }

    /// Runs a depth-limited subgame solve from the recall's current
    /// decision and returns the refined policy as an [`ApiSolved`]
    /// envelope. Expensive (seconds-scale); memoized via `cache_or`
    /// keyed by `(Witness, Kind::Depth)`.
    pub async fn solve_depth(&self, recall: Witness) -> anyhow::Result<ApiSolved> {
        self.solve(recall, Kind::Depth).await
    }

    /// Runs a safe (world-partitioned) subgame solve to terminal nodes.
    pub async fn solve_world(&self, recall: Witness) -> anyhow::Result<ApiSolved> {
        self.solve(recall, Kind::World).await
    }

    /// Runs a combined safe + depth-limited subgame solve.
    pub async fn solve_full(&self, recall: Witness) -> anyhow::Result<ApiSolved> {
        self.solve(recall, Kind::Full).await
    }

    /// Common dispatch for the three subgame solvers. Validates the
    /// witness, requires an in-memory blueprint, runs the solve on a
    /// blocking thread, and wraps the result in an `ApiSolved`. The
    /// `Kind` selects which `flagship.adapt_*` is invoked.
    async fn solve(&self, recall: Witness, kind: Kind) -> anyhow::Result<ApiSolved> {
        let recall = recall.validate()?;
        let blueprint = self
            .blueprint
            .ok_or_else(|| anyhow::anyhow!("subgame solve requires in-memory blueprint"))?;
        let key = (recall.clone(), kind);
        Ok(cache_or(key, || async move {
            let recall_inner = recall.clone();
            tokio::task::spawn_blocking(move || run_solve(blueprint, &recall_inner, kind))
                .await
                .expect("solve task panicked")
        })
        .await)
    }

    /// Opponent's hole-card-level posterior range from hero's POV.
    pub fn range(&self, recall: Witness) -> anyhow::Result<ApiOpponentRange> {
        self.posterior(recall, holdem::Flagship::opponent_observations)
    }

    /// Hero's hole-card-level **signalled** range — the posterior an
    /// opponent could form over hero's hand from hero's observed action
    /// history. Same response shape as [`Self::range`] with hero/opponent
    /// roles swapped in the underlying reach computation.
    pub fn signalled(&self, recall: Witness) -> anyhow::Result<ApiOpponentRange> {
        self.posterior(recall, holdem::Flagship::signalled_observations)
    }

    /// Common shape for `/strategy/range` and `/strategy/signalled`:
    /// validate the witness, require an in-memory blueprint, and project
    /// the `(observation, probability)` stream into the API response.
    fn posterior<F>(&self, recall: Witness, compute: F) -> anyhow::Result<ApiOpponentRange>
    where
        F: FnOnce(&'static holdem::Flagship, &Witness) -> Vec<(kicker::Observation, pokerkit::Probability)>,
    {
        let recall = recall.validate_observation()?;
        let blueprint = self
            .blueprint
            .ok_or_else(|| anyhow::anyhow!("posterior endpoint requires in-memory blueprint"))?;
        Ok(ApiOpponentRange {
            entries: compute(blueprint, &recall)
                .into_iter()
                .map(|(obs, weight)| ApiRangeEntry { obs, weight })
                .collect(),
        })
    }

    /// Aggregate per-(street, edge) strategy frequency across the entire
    /// blueprint. Expensive — full table scan with a window function.
    /// Use sparingly; intended for diagnostics, not real-time UI.
    ///
    /// Returns two frequency views per edge:
    /// - `avg_freq`: arithmetic mean of per-decision strategy probability,
    ///   one weight per decision point. Treats rare and common decisions
    ///   equally, so the long-tail of low-visit nodes dominates.
    /// - `weighted_freq`: visit-weighted strategy probability, equivalent
    ///   to `Σ weight(edge) / Σ total_weight` across decisions where the
    ///   edge was available. Approximates the actual frequency of this
    ///   action in real games (proportional to reach probability).
    pub async fn grid_usage(&self) -> anyhow::Result<Vec<ApiGridUsage>> {
        static SQL: OnceLock<&str> = OnceLock::<&str>::new();
        let sql = *SQL.get_or_init(|| {
            leaked(format!(
                "WITH per_decision AS ( \
                   SELECT present, edge, weight, \
                          SUM(weight) OVER (PARTITION BY past, present, choices) AS dec_total \
                   FROM   {} \
                 ) \
                 SELECT (present::int >> 8)::SMALLINT AS street, \
                        edge, \
                        AVG(weight / NULLIF(dec_total, 0))::REAL AS avg_freq, \
                        (SUM(weight) / NULLIF(SUM(dec_total), 0))::REAL AS weighted_freq, \
                        COUNT(*) AS n_decisions_with_edge, \
                        COUNT(*) FILTER (WHERE weight / NULLIF(dec_total, 0) > 0.5) AS n_dominant \
                 FROM   per_decision \
                 GROUP  BY street, edge \
                 ORDER  BY street, avg_freq DESC",
                blueprint()
            ))
        });
        let rows = self
            .client
            .query(sql, &[])
            .await
            .map_err(|e| anyhow::anyhow!("aggregate grid usage: {e}"))?;
        Ok(rows
            .into_iter()
            .map(|r| ApiGridUsage {
                street: street_name(r.get::<_, i16>(0)).to_string(),
                edge: format!("{}", Edge::from(r.get::<_, i64>(1) as u64)),
                avg_freq: r.get::<_, f32>(2),
                weighted_freq: r.get::<_, f32>(3),
                n_decisions_with_edge: r.get::<_, i64>(4),
                n_dominant: r.get::<_, i64>(5),
            })
            .collect())
    }
}
