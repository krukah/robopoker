//! One-container runtime for slumbot benchmarks.
//!
//! A single process hydrates the blueprint and database once, then spawns
//! per-variant tokio tasks for each requested
//! [`Variant`](crate::variant::Variant). The session count for each
//! variant comes from a trailing `*N` on its token (see
//! [`variant`](crate::variant) grammar) and falls back to the global
//! `--sessions N` flag (default `1`) when absent — so fast DB-lookup
//! variants like `blueprint` can stay at 1 while CFR-thinking variants
//! saturate vCPUs. All sessions across all variants share:
//!
//! - `Arc<tokio_postgres::Client>` for hand/action inserts
//! - `&'static Flagship` for any subgame variant (any cell with `depth` or `world`)
//! - [`Throttle`](crate::Throttle) capping aggregate in-flight HTTP requests
//! - SIGTERM / `TRAIN_DURATION` interrupt signal via [`rbp_core::brb`]
//!
//! Each session owns its own `Player`, `Recorder` (own Room row), and
//! `slumbot::Client` (own auth token). Sessions of the same variant emit
//! metrics with identical labels (`variant`, `regime`, `task_id`)
//! so OTLP aggregates them — the variant's bb/100 panel stays correct
//! across session counts. The point of multi-session is filling
//! otherwise-idle CPU cores during a single Player's CFR-solve wait:
//! a 4-vCPU task running `depth*4` runs 4 concurrent CFR solves
//! at 100% CPU utilization vs. 25% with one session.
use crate::benchmark::*;
use crate::client::*;
use crate::mode::*;
use crate::recorder::*;
use rbp_core::Variant;
use rbp_gameroom::VariantExt;
use tracing::Instrument;

/// Parsed runtime configuration. `Runtime::from_args()` produces this by
/// reading CLI flags (so the Dockerfile CMD can pipe env vars through
/// shell substitution without touching Rust).
pub struct Runtime {
    variants: Vec<(Variant, usize)>,
    mode: Mode,
    max_inflight: usize,
}

impl Runtime {
    pub fn new(
        variants: &str,
        hands: usize,
        continuous: bool,
        max_inflight: usize,
        default_sessions: usize,
    ) -> Self {
        let parsed = parse_list(variants);
        if parsed.is_empty() {
            eprintln!(
                "usage: slumbot --variants=a,b,c [--hands N] [--continuous] [--throttle N] [--sessions N]"
            );
            eprintln!("       grammar: 8 hypercube cells + `fish`");
            eprintln!("         fish | base | depth | world | dirac |");
            eprintln!("         depth+world | depth+dirac | world+dirac | depth+world+dirac");
            eprintln!("       (`+`-joined flags from {{depth, world, dirac}} in canonical order;");
            eprintln!("        `base` is the sentinel name for the empty flag-set cell)");
            eprintln!("       per-variant session override: trailing `*N` on a token");
            eprintln!("         example: base*1,dirac*1,depth+dirac*4,depth+world*4");
            eprintln!(
                "       --sessions: default session count when no `*N` suffix is given (default 1)."
            );
            eprintln!(
                "                   set to task vCPU count to saturate CPU during CFR think."
            );
            std::process::exit(1);
        }
        let default = default_sessions.max(1);
        let variants = parsed
            .into_iter()
            .map(|(v, n)| (v, n.unwrap_or(default).max(1)))
            .collect();
        Self {
            variants,
            mode: if continuous {
                Mode::Continuous
            } else {
                Mode::Fixed(hands)
            },
            max_inflight,
        }
    }

    pub async fn run(self) {
        tracing::info!(
            variants = %self.variants.iter().map(|(v, n)| format!("{}*{}", v.label(), n)).collect::<Vec<_>>().join(","),
            mode = ?self.mode,
            max_inflight = self.max_inflight,
            "slumbot runtime starting",
        );
        let db = connect().await;
        let flagship = if self.variants.iter().any(|(v, _)| v.requires_blueprint()) {
            Some(rbp_gameroom::hydrate_blueprint(db.clone()).await)
        } else {
            None
        };
        let throttle = Throttle::new(self.max_inflight);
        let handles: Vec<_> = self
            .variants
            .iter()
            .copied()
            .flat_map(|(v, n)| (0..n).map(move |i| (v, i)))
            .map(|(v, session)| {
                let db = db.clone();
                let throttle = throttle.clone();
                let mode = self.mode;
                tokio::spawn(
                    async move { execute(v, db, flagship, throttle, mode).await }
                        .instrument(tracing::info_span!("variant", name = v.label(), session)),
                )
            })
            .collect();
        for h in handles {
            h.await
                .inspect_err(|e| tracing::error!(error = %e, "variant task panicked"))
                .ok();
        }
    }
}

async fn connect() -> std::sync::Arc<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(
        &std::env::var("DB_URL").expect("DB_URL must be set"),
        tokio_postgres::NoTls,
    )
    .await
    .expect("database connection failed");
    tokio::spawn(async move {
        connection
            .await
            .inspect_err(|e| tracing::error!(error = %e, "database connection error"))
            .ok();
    });
    std::sync::Arc::new(client)
}

async fn execute(
    variant: Variant,
    db: std::sync::Arc<tokio_postgres::Client>,
    flagship: Option<&'static rbp_nlhe::Flagship>,
    throttle: Throttle,
    mode: Mode,
) {
    let mut player = variant.into_player(flagship);
    let mut recorder = Recorder::new(db, variant.id()).await;
    run_benchmark(variant, player.as_mut(), &mut recorder, throttle, mode).await;
}

async fn run_benchmark(
    variant: Variant,
    player: &mut dyn rbp_gameroom::Player,
    recorder: &mut Recorder,
    throttle: Throttle,
    mode: Mode,
) {
    let label = variant.label();
    tracing::info!(variant = label, ?mode, "benchmark starting");
    match mode {
        Mode::Fixed(hands) => {
            match Benchmark::run(variant, player, hands, recorder, throttle).await {
                Ok(bench) => bench.report(),
                Err(e) => tracing::error!(variant = label, error = %e, "benchmark failed"),
            }
        }
        Mode::Continuous => {
            Benchmark::continuous(variant, player, recorder, throttle)
                .await
                .report();
        }
    }
}

/// Parse a comma-separated list (the `--variants` argument value),
/// deduping by label. `Option<usize>` is the per-variant session-count
/// override extracted from a trailing `*N`. Prints to stderr and exits
/// on any unknown token or malformed `*N` suffix — runs at binary
/// startup, fail-fast.
fn parse_list(raw: &str) -> Vec<(Variant, Option<usize>)> {
    let mut out: Vec<(Variant, Option<usize>)> = Vec::new();
    for token in raw.split(',').filter(|t| !t.is_empty()) {
        let (token, sessions) = parse_session_suffix(token.trim());
        match Variant::parse(token) {
            Some(v)
                if !out
                    .iter()
                    .any(|(existing, _)| existing.label() == v.label()) =>
            {
                out.push((v, sessions))
            }
            Some(_) => {}
            None => {
                eprintln!("unknown variant: {token}");
                std::process::exit(1);
            }
        }
    }
    out
}

fn parse_session_suffix(token: &str) -> (&str, Option<usize>) {
    match token.rsplit_once('*') {
        Some((variant, count)) => match count.parse::<usize>() {
            Ok(n) if n >= 1 => (variant, Some(n)),
            _ => {
                eprintln!("invalid session count in token (expected `*N` with N>=1): {token}");
                std::process::exit(1);
            }
        },
        None => (token, None),
    }
}
