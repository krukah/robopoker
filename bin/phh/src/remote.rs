//! A running backend as an [`Oracle`], for duelling without database credentials.
//!
//! [`crate::mind`] is the fast path: an in-memory blueprint answers a lookup in
//! microseconds. It needs `$DB_URL`, and the production database is private.
//! This is the slow path that needs nothing but a URL — the same endpoint
//! `agree` already uses, with the two things a rollout cannot do without:
//!
//! - **A cache.** Information sets repeat heavily across worlds and across
//!   spots, and every hit is a round trip saved.
//! - **Concurrency.** A rollout's own lookups are strictly sequential, so the
//!   only axis left is spots. Plain threads rather than tasks, because
//!   [`Oracle`] is synchronous by design and a thread outside the runtime may
//!   block on it.

use kicker::Edge;
use kicker::Witness;
use kicker::dto::GetPolicy;
use phh::Oracle;
use phh::Policy;
use pokerkit::Config;
use pokerkit::Probability;
use pokerkit::Variant;
use std::collections::HashMap;
use std::sync::Mutex;

/// Threads issuing blueprint lookups. The backend answers in a couple of
/// milliseconds and the wall clock is round trips; past this a shared
/// deployment starts to notice.
const LANES: usize = 32;

/// Threads issuing subgame solves. Each one burns roughly a second of backend
/// CPU rather than a map read, so this is a courtesy limit on a deployment that
/// is also serving the analysis UI — not a throughput choice.
const SOLVE_LANES: usize = 8;

/// The strategy route a variant maps onto.
///
/// The backend already exposes the subgame solvers, so scoring a `depth` or
/// `world` bot needs no database — only patience. `/policy` is a map lookup;
/// the other three run a real CFR re-solve against a wall-clock budget and cost
/// about a second each, of *backend* CPU.
/// The `dirac` axis has no route of its own — it is an output transform the
/// backend does not apply — so a `dirac` variant scores as its plain twin here.
/// Say so rather than let the flag look like it did something.
#[rustfmt::skip]
pub fn endpoint(variant: Variant) -> Option<&'static str> {
    let Config { depth, world, dirac } = variant.config()?;
    if dirac {
        eprintln!("note: /strategy has no dirac route — scoring `{}` without it", variant.label());
    }
    Some(match (depth, world) {
        (false, false) => "policy",
        (true,  false) => "depth",
        (false, true ) => "world",
        (true,  true ) => "full",
    })
}

/// The strategy API, with the round trips memoized.
pub struct Remote {
    client: reqwest::Client,
    handle: tokio::runtime::Handle,
    cache: Mutex<HashMap<String, Policy>>,
    api: String,
    route: &'static str,
}

impl Remote {
    pub fn new(api: &str, route: &'static str) -> Self {
        Self {
            client: reqwest::Client::new(),
            handle: tokio::runtime::Handle::current(),
            cache: Mutex::new(HashMap::new()),
            api: api.to_string(),
            route,
        }
    }

    /// Whether every lookup costs a CFR re-solve rather than a map read.
    pub fn solves(&self) -> bool {
        self.route != "policy"
    }

    /// Queries answered from cache rather than over the wire.
    pub fn hits(&self) -> usize {
        self.cache.lock().map_or(0, |c| c.len())
    }

    async fn ask(&self, query: &GetPolicy) -> anyhow::Result<Policy> {
        Ok(self
            .client
            .post(format!("{}/strategy/{}", self.api, self.route))
            .json(query)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?
            .get("policy")
            .and_then(|p| p.get("accumulated"))
            .and_then(serde_json::Value::as_object)
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| Some((Edge::try_from(k.as_str()).ok()?, v.as_f64()? as Probability)))
                    .collect::<Policy>()
            })
            .unwrap_or_default())
    }
}

impl Oracle for Remote {
    /// A failed request is indistinguishable from an unreached information set
    /// here, and both are counted the same way — as a miss on the [`phh::Wager`]
    /// that asked. The report prints the total.
    fn policy(&self, witness: &Witness) -> Policy {
        let query = GetPolicy::from(witness);
        let Ok(key) = serde_json::to_string(&query) else {
            return Policy::default();
        };
        if let Some(policy) = self.cache.lock().ok().and_then(|c| c.get(&key).cloned()) {
            return policy;
        }
        let policy = self.handle.block_on(self.ask(&query)).unwrap_or_default();
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(key, policy.clone());
        }
        policy
    }
}

/// Duels `spots` across [`LANES`] threads, since each one's lookups are serial.
///
/// Runs on plain OS threads rather than tokio tasks: [`Oracle::policy`] is
/// synchronous, and blocking on a runtime handle is only legal from a thread the
/// runtime does not own.
pub fn ledger(remote: &Remote, spots: &[phh::Spot], worlds: usize, seed: u64) -> phh::Ledger {
    let lanes = if remote.solves() { SOLVE_LANES } else { LANES }.min(spots.len().max(1));
    std::thread::scope(|scope| {
        spots
            .chunks(spots.len().div_ceil(lanes))
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .filter_map(|spot| phh::duel(&remote, spot, &phh::worlds(spot, worlds, seed), seed))
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|lane| lane.join().ok())
            .flatten()
            .collect()
    })
}
