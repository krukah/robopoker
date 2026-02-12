//! Core type aliases, traits, and constants for robopoker.
//!
//! This crate provides the foundational types and configuration parameters
//! used throughout the robopoker workspace.
#![allow(dead_code)]

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
// IDENTITY TYPES
// ============================================================================
use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;

/// Generic ID wrapper providing compile-time type safety over uuid::Uuid.
pub struct ID<T> {
    inner: uuid::Uuid,
    marker: PhantomData<T>,
}

impl<T> ID<T> {
    pub fn inner(&self) -> uuid::Uuid {
        self.inner
    }
    /// Cast ID<T> to ID<U> while preserving the underlying UUID.
    /// Useful for converting between marker types.
    pub fn cast<U>(self) -> ID<U> {
        ID {
            inner: self.inner,
            marker: PhantomData,
        }
    }
}

impl<T> From<ID<T>> for uuid::Uuid {
    fn from(id: ID<T>) -> Self {
        id.inner()
    }
}
impl<T> From<uuid::Uuid> for ID<T> {
    fn from(inner: uuid::Uuid) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<T> Default for ID<T> {
    fn default() -> Self {
        Self {
            inner: uuid::Uuid::now_v7(),
            marker: PhantomData,
        }
    }
}

impl<T> Copy for ID<T> {}
impl<T> Clone for ID<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for ID<T> {}
impl<T> PartialEq for ID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Ord for ID<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}
impl<T> PartialOrd for ID<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Hash for ID<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.inner.hash(state);
    }
}

impl<T> Debug for ID<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ID").field(&self.inner).finish()
    }
}
impl<T> Display for ID<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

// ============================================================================
// GAME TREE PARAMETERS
// ============================================================================
/// Number of players at the table.
pub const N: usize = 2;
/// Starting stack size in big blinds.
pub const STACK: Chips = 100;
/// Big blind amount.
pub const B_BLIND: Chips = 2;
/// Small blind amount.
pub const S_BLIND: Chips = 1;
/// Maximum re-raises per betting round (limits tree width).
pub const MAX_RAISE_REPEATS: usize = 3;
/// Maximum tree depth for real-time subgame solving.
pub const MAX_DEPTH_SUBGAME: usize = 16;
/// Maximum tree depth for full game abstraction.
pub const MAX_DEPTH_ALLGAME: usize = 32;

/// Timeout for voluntary card reveal at showdown (seconds).
pub const SHOWDOWN_TIMEOUT: u64 = 5;

// ============================================================================
// SINKHORN OPTIMAL TRANSPORT
// Entropy-regularized EMD for comparing hand distributions across abstractions.
// ============================================================================
/// Entropy regularization strength. Lower = closer to true EMD, higher = faster convergence.
pub const SINKHORN_TEMPERATURE: Entropy = 0.025;
/// Maximum Sinkhorn-Knopp iterations before stopping.
pub const SINKHORN_ITERATIONS: usize = 128;
/// Early stopping threshold on marginal constraint violation.
pub const SINKHORN_TOLERANCE: Energy = 0.001;

// ============================================================================
// K-MEANS CLUSTERING
// Hierarchical abstraction: river equity → turn clusters → flop clusters.
// ============================================================================
/// Lloyd's algorithm iterations for flop clustering.
pub const KMEANS_FLOP_TRAINING_ITERATIONS: usize = 20;
/// Lloyd's algorithm iterations for turn clustering.
pub const KMEANS_TURN_TRAINING_ITERATIONS: usize = 24;
/// Number of flop buckets (distributions over turn clusters).
pub const KMEANS_FLOP_CLUSTER_COUNT: usize = 128;
/// Number of turn buckets (distributions over river equity).
pub const KMEANS_TURN_CLUSTER_COUNT: usize = 144;
/// Equity histogram resolution (0%, 1%, ..., 100%).
pub const KMEANS_EQTY_CLUSTER_COUNT: usize = 101;

// ============================================================================
// MCCFR SOLVER CONFIGURATIONS
// Batch size = trees per iteration, tree count = total training budget.
// ============================================================================
/// Asymmetric payoff for RPS test game (rock beats scissors by 2x).
pub const ASYMMETRIC_UTILITY: f32 = 2.0;
/// Trees sampled per RPS iteration.
pub const CFR_BATCH_SIZE_RPS: usize = 1;
/// Total RPS training budget (small game converges fast).
pub const CFR_TREE_COUNT_RPS: usize = 8192;
/// Trees sampled per NLHE iteration (parallelized across threads).
pub const CFR_BATCH_SIZE_NLHE: usize = 128;
/// Total NLHE training budget (~268M trees for production).
pub const CFR_TREE_COUNT_NLHE: usize = 0x10000000;
/// Trees sampled per river-only iteration (testing/debugging).
pub const CFR_BATCH_SIZE_RIVER: usize = 16;
/// River-only training budget (~65K trees).
pub const CFR_TREE_COUNT_RIVER: usize = 0x10000;

// ============================================================================
// AVERAGE STRATEGY SAMPLING
// Biased sampling from cumulative policy: σ'(a) = max(ε, (τ·σ(a) + β) / (Σσ + β))
// ============================================================================
/// Temperature (T) - controls sampling entropy via policy scaling.
/// Higher T → more uniform (exploratory); lower T → more peaked (greedy).
/// Formula: σ'(a) = max(ε, (σ(a)/T + β) / (Σσ + β)).
pub const SAMPLING_TEMPERATURE: Entropy = 2.0;
/// Smoothing (β) - pseudocount added to numerator and denominator.
/// Higher values pull sampling toward uniform (maximum entropy prior).
pub const SAMPLING_SMOOTHING: Energy = 0.5;
/// Epsilon (ε) - minimum sampling probability floor.
/// Ensures every action retains at least ε probability for exploration.
pub const SAMPLING_CURIOSITY: Probability = 0.01;

// ============================================================================
// REGRET MATCHING
// Convert cumulative regrets to current iteration strategy via normalization.
// ============================================================================
/// Minimum policy weight to prevent division by zero in normalization.
pub const POLICY_MIN: Probability = Probability::MIN_POSITIVE;
/// Floor for cumulative regret storage (prevents unbounded negative growth).
pub const REGRET_MIN: Utility = -4e6;

// ============================================================================
// PROBABILISTIC PRUNING (see `mccfr::PluribusSampling`)
// Skip sampling low-regret actions to accelerate convergence.
// PRUNING_THRESHOLD > REGRET_MIN so floored actions can recover via exploration.
// ============================================================================
/// Actions with regret below this are candidates for pruning (-300k ≈ 3× max pot).
pub const PRUNING_THRESHOLD: Utility = -3e5;
/// Probability of sampling pruned actions anyway (prevents permanent lock-out).
pub const PRUNING_EXPLORE: Probability = 0.05;
/// Warm-up epochs before pruning activates (let regrets stabilize first).
pub const PRUNING_WARMUP: usize = 524288;

// ============================================================================
// SUBGAME SOLVING (see `mccfr::subgame`)
// Real-time refinement of blueprint strategy at decision points.
// ============================================================================
/// Alternative hands in the gadget game (Pluribus uses 4).
pub const SUBGAME_ALTS: usize = 4;
/// CFR iterations for real-time subgame refinement.
pub const SUBGAME_ITERATIONS: usize = 1024;

// ============================================================================
// TRAINING INFRASTRUCTURE
// ============================================================================
/// Interval between progress log messages during training.
pub const TRAINING_LOG_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

// ============================================================================
// REGRET INITIALIZATION BIAS
// Weights (not probabilities) for initial regret seeding. Only ratios matter.
// With k=4 raises: p(fold)≈50%, p(raises)≈33%, p(other)≈17%.
// ============================================================================
/// Initial regret weight for fold actions (high = fold more often early).
pub const BIAS_FOLDS: Utility = 3.0;
/// Initial regret weight for raise actions (low = raise less often early).
pub const BIAS_RAISE: Utility = 0.5;
/// Initial regret weight for call/check actions (baseline).
pub const BIAS_OTHER: Utility = 1.0;

// ============================================================================
// RUNTIME UTILITIES
// ============================================================================
/// Initialize dual logging (terminal + file) with timestamped log files.
/// Creates `logs/` directory and writes DEBUG level to file, INFO to terminal.
#[cfg(feature = "server")]
pub fn log() {
    std::fs::create_dir_all("logs").expect("create logs directory");
    let config = simplelog::ConfigBuilder::new()
        .set_location_level(log::LevelFilter::Off)
        .set_target_level(log::LevelFilter::Off)
        .set_thread_level(log::LevelFilter::Off)
        .build();
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time moves slow")
        .as_secs();
    let file = simplelog::WriteLogger::new(
        log::LevelFilter::Debug,
        config.clone(),
        std::fs::File::create(format!("logs/{}.log", time)).expect("create log file"),
    );
    let term = simplelog::TermLogger::new(
        log::LevelFilter::Info,
        config.clone(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    );
    simplelog::CombinedLogger::init(vec![term, file]).expect("initialize logger");
}

/// Register Ctrl+C handler for immediate (non-graceful) termination.
/// Use when you need hard shutdown without waiting for current batch.
#[cfg(feature = "server")]
pub fn kys() {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!();
        log::warn!("violent interrupt received, exiting immediately");
        std::process::exit(0);
    });
}

/// Global interrupt flag for graceful shutdown coordination.
#[cfg(feature = "server")]
static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Optional training deadline from TRAIN_DURATION env var.
#[cfg(feature = "server")]
static DEADLINE: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
/// Check if graceful shutdown was requested (via stdin "Q") or deadline reached.
#[cfg(feature = "server")]
pub fn interrupted() -> bool {
    INTERRUPTED.load(std::sync::atomic::Ordering::Relaxed)
        || DEADLINE
            .get()
            .map_or(false, |d| std::time::Instant::now() >= *d)
}
/// No-op interrupt check when server feature disabled.
#[cfg(not(feature = "server"))]
pub fn interrupted() -> bool {
    false
}
/// Register graceful interrupt handler. Type "Q" + Enter to stop after current batch.
/// Optionally set TRAIN_DURATION env var (e.g., "2h", "30m") for timed runs.
#[cfg(feature = "server")]
pub fn brb() {
    if let Ok(duration) = std::env::var("TRAIN_DURATION") {
        if let Some(deadline) = parse_duration(&duration) {
            let _ = DEADLINE.set(std::time::Instant::now() + deadline);
            log::info!("training will stop after {}", duration);
        }
    }
    std::thread::spawn(|| {
        loop {
            let ref mut buffer = String::new();
            if let Ok(_) = std::io::stdin().read_line(buffer) {
                if buffer.trim().to_uppercase() == "Q" {
                    log::warn!("graceful interrupt requested, finishing current batch...");
                    INTERRUPTED.store(true, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
            }
        }
    });
}
/// Parse duration string like "30s", "5m", "2h", "1d" into Duration.
#[cfg(feature = "server")]
fn parse_duration(s: &str) -> Option<std::time::Duration> {
    let s = s.trim();
    let (num, unit) = s.split_at(s.len().saturating_sub(1));
    let value: u64 = num.parse().ok()?;
    match unit {
        "s" => Some(std::time::Duration::from_secs(value)),
        "m" => Some(std::time::Duration::from_secs(value * 60)),
        "h" => Some(std::time::Duration::from_secs(value * 3600)),
        "d" => Some(std::time::Duration::from_secs(value * 86400)),
        _ => None,
    }
}
