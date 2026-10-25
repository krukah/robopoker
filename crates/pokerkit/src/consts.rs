#![allow(dead_code)]

use super::*;

// GAME TREE PARAMETERS
#[cfg(not(feature = "sixmax"))]
pub const N: usize = 2;

#[cfg(feature = "sixmax")]
pub const N: usize = 6;

pub const STACK: Chips = 200;

pub const B_BLIND: Chips = 2;

pub const S_BLIND: Chips = 1;

pub const MAX_RAISE_REPEATS: usize = 3;

/// Off-tree snap tolerance for [`Translation::Exact`], as a **relative** difference in [0, 1].
pub const EXACT_SNAP_TOLERANCE: f64 = 0.05;

/// Words in a subgame-carrying `Path<W>`: 1 heads-up (byte-identical to the historical u64 Path
/// and its `BIGINT` column), 4 multiway, where a legal street can exceed 12 edges.
pub const WORDS: usize = if N > 2 { 4 } else { 1 };

/// Maximum edges in one packed Path word (12 edges × 5 bits = 60 ≤ 64 bits).
pub const MAX_PATH_EDGES: usize = 12;

// BET SIZING ABSTRACTION RAISES is the canonical pool; the index arrays below select subsets.
// To change the game tree, edit RAISES or those arrays.
pub const OPENS: [Chips; 4] = [2, 3, 4, 5];

/// Canonical raise pool as pot-relative (numerator, denominator) fractions.
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

#[rustfmt::skip]
pub const PLURIBUS_INDICES: [&[usize]; 12] = [
    &[],              // (Pref, 0) opens — see OPENS
    &[5, 8, 9],       // (Pref, 1) 3-bet:   [1:1, 2:1, 3:1]
    &[5, 8],          // (Pref, N) 4-bet+:  [1:1, 2:1]
    &[0, 2, 5, 8],    // (Flop, 0):         [1/4, 1/2, 1:1, 2:1]
    &[2, 5],          // (Flop, 1):         [1/2, 1:1]
    &[5],             // (Flop, N):         [1:1]
    &[1, 2, 5, 8],    // (Turn, 0):         [1/3, 1/2, 1:1, 2:1]
    &[5, 8],          // (Turn, 1):         [1:1, 2:1]
    &[5],             // (Turn, N):         [1:1]
    &[1, 2, 5, 8],    // (Rive, 0):         [1/3, 1/2, 1:1, 2:1]
    &[5, 8],          // (Rive, 1):         [1:1, 2:1]
    &[5],             // (Rive, N):         [1:1]
];

// Slumbot regime: uniform grid (½ pot, full pot) at every street/depth. UI also offers Min Bet
// and All In, handled by Edge::Raise (min coercion) and Edge::Shove respectively.
pub const SLUMBOT_INDICES: &[usize] = &[2, 5];

// GAME PACING (milliseconds, at `Tempo::Table`). Beats, not delays: each is
// a state change a player should perceive separately.
pub const PACE_DEAL_HOLE: u64 = 300;
pub const PACE_DEAL_BOARD: u64 = 450;
/// Think time for bot actions, drawn uniformly from this range.
///
/// Flat by construction: the draw is independent of the decision, so a long
/// pause is never a tell. Correlating it with hand strength would read as
/// warmer and leak the thing the bot is trying not to give away.
pub const PACE_BOT_THINK_MIN: u64 = 500;
pub const PACE_BOT_THINK_MAX: u64 = 1500;
/// Window for voluntary card reveals at showdown.
pub const PACE_SHOWDOWN: u64 = 1200;
/// Pause between hands (after settlement, before next deal).
pub const PACE_RESULTS: u64 = 2500;
/// Timeout for human player decisions. Not scaled by [`Tempo`] — this is
/// the player's own clock, and it should not move when the table's does.
pub const PACE_DECISION: u64 = 10000;
/// Timeout for room startup (waiting for WebSocket connection).
pub const PACE_ROOM_STARTUP: u64 = 30000;

/// Maximum consecutive all-timeout hands before ending session.
pub const MAX_IDLE_HANDS: usize = 3;

// K-MEANS CLUSTERING — STRUCTURAL CONSTANTS Cluster counts are const-generic / array-size;
// can't be runtime config.
const _: () = assert!(KMEANS_FLOP_CLUSTER_COUNT <= KMEANS_MAX_CLUSTER_COUNT);

const _: () = assert!(KMEANS_TURN_CLUSTER_COUNT <= KMEANS_MAX_CLUSTER_COUNT);

const _: () = assert!(KMEANS_EQTY_CLUSTER_COUNT <= KMEANS_MAX_CLUSTER_COUNT);

pub const KMEANS_MAX_CLUSTER_COUNT: usize = 256;

/// Number of flop buckets (distributions over turn clusters).
pub const KMEANS_FLOP_CLUSTER_COUNT: usize = 256;

/// Number of turn buckets (distributions over river equity).
pub const KMEANS_TURN_CLUSTER_COUNT: usize = 256;

/// Equity histogram resolution (0%, 1%, ..., 100%).
pub const KMEANS_EQTY_CLUSTER_COUNT: usize = 101;

/// Asymmetric payoff for RPS test game (rock beats scissors by 2x).
pub const ASYMMETRIC_UTILITY: f32 = 2.0;

/// Minimum policy weight to prevent division by zero in normalization.
pub const EPSILON: Probability = Probability::MIN_POSITIVE;

// SUBGAME SOLVING — STRUCTURAL CONSTANTS Const-generic depths in the world / depth solvers;
// can't be runtime config. Tuning knobs live in `SubgameHyperParams` / `FrontierHyperParams`.
pub const N_WORLDS: usize = 4;

/// Number of biased continuation strategies at depth-limited frontiers.
pub const FRONTIER_LEAVES: usize = 4;

#[cfg(feature = "server")]
pub(crate) static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Optional training deadline from TRAIN_DURATION env var.
#[cfg(feature = "server")]
pub(crate) static DEADLINE: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::<std::time::Instant>::new();
