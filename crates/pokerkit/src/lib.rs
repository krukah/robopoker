//! Core type aliases, traits, and constants for robopoker.
//!
//! This crate provides the foundational types and configuration parameters
//! used throughout the robopoker workspace.
#![allow(dead_code)]

mod id;
mod macros;
mod metrics;
mod regime;
mod translate;
mod translation;
mod variant;
mod version;

pub use id::*;
pub use metrics::*;
pub use regime::*;
pub use translate::*;
pub use translation::*;
pub use variant::*;
pub use version::*;

// ============================================================================
// TYPE ALIASES
// ============================================================================
/// Stack sizes and bet amounts in big blinds.
pub type Chips = i16;
/// Seat index around the table (0 = button in heads-up).
pub type Position = usize;
/// Training iteration counter for CFR epochs.
pub type Epoch = i16;
/// Distance metrics, convergence thresholds, and smoothing terms.
pub type Energy = f32;
/// Temperature parameters and information-theoretic measures.
pub type Entropy = f32;
/// Expected values, regrets, and payoffs.
pub type Utility = f32;
/// Strategy weights, sampling distributions, and reach probabilities.
pub type Probability = f32;

// ============================================================================
// TRAITS
// ============================================================================
/// Random instance generation for testing and Monte Carlo sampling.
pub trait Arbitrary {
    /// Generate a uniformly random instance.
    fn random() -> Self;
}

/// Unique identifier trait for domain entities.
pub trait Unique<T = Self> {
    fn id(&self) -> ID<T>;
}

// ============================================================================
// GAME TREE PARAMETERS
// ============================================================================
/// Number of players at the table.
pub const N: usize = 2;
/// Starting stack size in chips.
pub const STACK: Chips = 200;
/// Big blind amount.
pub const B_BLIND: Chips = 2;
/// Small blind amount.
pub const S_BLIND: Chips = 1;
/// Maximum re-raises per betting round (limits tree width).
pub const MAX_RAISE_REPEATS: usize = 3;
/// Maximum edges in a packed Path (12 nibbles × 5 bits = 60 bits ≤ 64 bits).
/// Data-representation limit, not a solver depth knob — the subgame tree's
/// effective depth is controlled by where `DepthGame::at_frontier` fires
/// (first chance node past origin), not by this constant.
pub const MAX_PATH_EDGES: usize = 12;

// ============================================================================
// BET SIZING ABSTRACTION
// RAISES is the canonical pool; SIZE_* select subsets via index.
// To change the game tree, edit RAISES or the index arrays below.
// ============================================================================
/// Preflop open sizes in BB units (depth=0 only).
pub const OPENS: [Chips; 4] = [2, 3, 4, 5];
/// Canonical raise pool as pot-relative (numerator, denominator) fractions.
/// Index position = u8 encoding offset from 10. 1:1 with Odds::GRID.
///   0     1     2     3     4     5     6     7     8     9
///  25%   33%   50%   67%   75%  100%  125%  150%  200%  300%
pub const RAISES: [(Chips, Chips); 10] = [
    (1, 4),
    (1, 3),
    (1, 2),
    (2, 3),
    (3, 4),
    (1, 1),
    (5, 4),
    (3, 2),
    (2, 1),
    (3, 1),
];
const fn pick<const N: usize>(idx: [usize; N]) -> [(Chips, Chips); N] {
    let mut r = [(0, 0); N];
    let mut i = 0;
    while i < N {
        r[i] = RAISES[idx[i]];
        i += 1;
    }
    r
}

/// Action grid for the Pluribus regime. Pluribus-faithful (Brown &
/// Sandholm 2019) widths on first-bet rows, shrunk on subsequent raises.
/// No SPR axis on the menu — pot-relative sizing already self-scales
/// with stack depth (1× pot at SPR=1 *is* all-in), so the menu is keyed
/// only on `(street, depth)`. All-in remains available at every node as
/// `Edge::Shove`.
///
/// **Cells are indices into `RAISES`:**
/// `0=1/4  1=1/3  2=1/2  3=2/3  4=3/4  5=1:1  6=5/4  7=3/2  8=2:1  9=3:1`
///
/// **Row layout:** `street * 3 + min(depth, 2)`, so depth ≥ 2 collapses
/// to the "N" row:
///
/// | row | cell | row | cell | row | cell | row  | cell |
/// |-----|------|-----|------|-----|------|------|------|
/// | 0   | Pref/0 (opens) | 3 | Flop/0 | 6 | Turn/0 | 9  | Rive/0 |
/// | 1   | Pref/1 (3-bet) | 4 | Flop/1 | 7 | Turn/1 | 10 | Rive/1 |
/// | 2   | Pref/N (4-bet+)| 5 | Flop/N | 8 | Turn/N | 11 | Rive/N |
///
/// `(Pref, 0)` is empty here — preflop opens are BB-relative and use
/// `OPENS` instead.
///
/// **Bit-packing budget:** max cell width is 5 (Flop/0:
/// `[1/4, 1/2, 3/4, 1:1, 2:1]`). Max `choices` is 5 raises +
/// Fold/Check/Call/Shove = 9 edges × 5 bits = 45 bits, under the
/// 60-bit Path capacity.
#[rustfmt::skip]
pub const PLURIBUS_INDICES: [&[usize]; 12] = [
    &[],              // (Pref, 0) opens — see OPENS
    &[5, 8],          // (Pref, 1) 3-bet:   [1:1, 2:1]
    &[5],             // (Pref, N) 4-bet+:  [1:1]
    &[0, 2, 4, 5, 8], // (Flop, 0):         [1/4, 1/2, 3/4, 1:1, 2:1]
    &[2, 5],          // (Flop, 1):         [1/2, 1:1]
    &[5],             // (Flop, N):         [1:1]
    &[1, 2, 5, 8],    // (Turn, 0):         [1/3, 1/2, 1:1, 2:1]
    &[5, 8],          // (Turn, 1):         [1:1, 2:1]
    &[5],             // (Turn, N):         [1:1]
    &[1, 2, 5, 8],    // (Rive, 0):         [1/3, 1/2, 1:1, 2:1]
    &[5, 8],          // (Rive, 1):         [1:1, 2:1]
    &[5],             // (Rive, N):         [1:1]
];

// Slumbot regime: uniform grid (½ pot, full pot) at every street/depth.
// UI also offers Min Bet and All In, handled by Edge::Raise (min coercion)
// and Edge::Shove respectively.
pub const SLUMBOT_INDICES: &[usize] = &[2, 5];

// GAME PACING (milliseconds)
/// Delay after hand start before dealing hole cards.
pub const PACE_DEAL_HOLE: u64 = 0;
/// Delay after dealing community cards.
pub const PACE_DEAL_BOARD: u64 = 0;
/// Simulated think time for bot actions.
pub const PACE_BOT_THINK: u64 = 0;
/// Window for voluntary card reveals at showdown.
pub const PACE_SHOWDOWN: u64 = 0;
/// Pause between hands (after settlement, before next deal).
pub const PACE_RESULTS: u64 = 4000;
/// Timeout for human player decisions.
pub const PACE_DECISION: u64 = 10000;
/// Timeout for room startup (waiting for WebSocket connection).
pub const PACE_ROOM_STARTUP: u64 = 30000;
/// Maximum consecutive all-timeout hands before ending session.
pub const MAX_IDLE_HANDS: usize = 3;

// ============================================================================
// K-MEANS CLUSTERING — STRUCTURAL CONSTANTS
// Cluster counts are const-generic / array-size; can't be runtime config.
// Tuning knobs (iterations, RMS interval, drift threshold) live in
// `KmeansHyperParams` (lloyd); Sinkhorn knobs in
// `SinkhornHyperParams`.
// ============================================================================
const _: () = assert!(KMEANS_FLOP_CLUSTER_COUNT <= KMEANS_MAX_CLUSTER_COUNT);
const _: () = assert!(KMEANS_TURN_CLUSTER_COUNT <= KMEANS_MAX_CLUSTER_COUNT);
const _: () = assert!(KMEANS_EQTY_CLUSTER_COUNT <= KMEANS_MAX_CLUSTER_COUNT);
/// Maximum clusters per street. Bound by Abstraction's 8-bit index field
/// (0..=255 = 256 distinct values).
pub const KMEANS_MAX_CLUSTER_COUNT: usize = 256;
/// Number of flop buckets (distributions over turn clusters).
pub const KMEANS_FLOP_CLUSTER_COUNT: usize = 256;
/// Number of turn buckets (distributions over river equity).
pub const KMEANS_TURN_CLUSTER_COUNT: usize = 256;
/// Equity histogram resolution (0%, 1%, ..., 100%).
pub const KMEANS_EQTY_CLUSTER_COUNT: usize = 101;

// ============================================================================
// MCCFR SOLVER CONFIGURATIONS
// Batch size = trees per iteration, tree count = total training budget.
// ============================================================================
/// Asymmetric payoff for RPS test game (rock beats scissors by 2x).
pub const ASYMMETRIC_UTILITY: f32 = 2.0;

// ============================================================================
// REGRET MATCHING
// ============================================================================
/// Minimum policy weight to prevent division by zero in normalization.
pub const EPSILON: Probability = Probability::MIN_POSITIVE;

// ============================================================================
// SUBGAME SOLVING — STRUCTURAL CONSTANTS
// `N_WORLDS` and `FRONTIER_LEAVES` are const-generic depths in the world /
// depth solvers; they can't be runtime config. Tuning knobs live in
// `SubgameHyperParams` (subgame) and `FrontierHyperParams` (horizon).
// ============================================================================
/// Alternative opponent hand partitions in the subgame (safe subgame solving).
/// Each world represents a partition of the opponent's range conditioned on
/// the observed action sequence. More worlds = finer range partitioning =
/// more robust strategy, but higher memory and slower convergence.
pub const N_WORLDS: usize = 4;
/// Number of biased continuation strategies at depth-limited frontiers.
/// D=4: unmodified blueprint + fold-biased + call-biased + raise-biased (Pluribus).
pub const FRONTIER_LEAVES: usize = 4;

// ============================================================================
// RUNTIME UTILITIES
// ============================================================================
/// Register Ctrl+C handler for immediate (non-graceful) termination.
/// Use when you need hard shutdown without waiting for current batch.
#[cfg(feature = "server")]
pub fn kys() {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!();
        tracing::warn!("violent interrupt received, exiting immediately");
        std::process::exit(0);
    });
}

/// Global interrupt flag for graceful shutdown coordination.
#[cfg(feature = "server")]
static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Optional training deadline from TRAIN_DURATION env var.
#[cfg(feature = "server")]
static DEADLINE: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::<std::time::Instant>::new();
/// Check if graceful shutdown was requested (via stdin "Q") or deadline reached.
#[cfg(feature = "server")]
pub fn interrupted() -> bool {
    INTERRUPTED.load(std::sync::atomic::Ordering::Relaxed)
        || DEADLINE.get().is_some_and(|d| std::time::Instant::now() >= *d)
}
/// No-op interrupt check when server feature disabled.
#[cfg(not(feature = "server"))]
pub fn interrupted() -> bool {
    false
}
/// Register graceful interrupt handler. Type "Q" + Enter to stop after current batch.
/// Also handles SIGTERM (sent by ECS stop-task) for graceful remote shutdown.
/// Optionally set TRAIN_DURATION env var (e.g., "2h", "30m") for timed runs.
#[cfg(feature = "server")]
pub fn brb() {
    std::env::var("TRAIN_DURATION")
        .ok()
        .and_then(|d| parse_duration(&d).map(|dur| (d, dur)))
        .inspect(|(s, dur)| {
            let _ = DEADLINE.set(std::time::Instant::now() + *dur);
            tracing::info!(duration = %s, "training will stop after duration");
        });
    tokio::spawn(async move {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to register SIGTERM handler")
            .recv()
            .await;
        tracing::warn!("SIGTERM received, finishing current batch...");
        INTERRUPTED.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    std::thread::spawn(|| {
        loop {
            let ref mut buffer = String::new();
            if std::io::stdin().read_line(buffer).is_ok() && buffer.trim().eq_ignore_ascii_case("Q") {
                tracing::warn!("graceful interrupt requested, finishing current batch...");
                INTERRUPTED.store(true, std::sync::atomic::Ordering::Relaxed);
                break;
            }
        }
    });
}
/// Parse duration string like "30s", "5m", "2h", "1d" into Duration.
#[cfg(feature = "server")]
fn parse_duration(s: &str) -> Option<std::time::Duration> {
    let s = s.trim();
    let (num, unit) = s.split_at(s.len().saturating_sub(1));
    let value: u64 = num
        .parse()
        .inspect_err(|e| tracing::warn!(input = %num, error = %e, "parse_duration: number parse failed"))
        .ok()?;
    match unit {
        "s" => Some(std::time::Duration::from_secs(value)),
        "m" => Some(std::time::Duration::from_secs(value * 60)),
        "h" => Some(std::time::Duration::from_secs(value * 3600)),
        "d" => Some(std::time::Duration::from_secs(value * 86400)),
        _ => None,
    }
}
