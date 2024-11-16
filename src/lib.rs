pub mod cards;
pub mod clustering;
pub mod kmeans;
pub mod mccfr;
pub mod play;
pub mod players;
pub mod rts;
pub mod transport;

type Chips = i16;
type Equity = f32;
type Utility = f32;
type Probability = f32;

// game tree parameters
const N: usize = 2;
const STACK: Chips = 100;
const B_BLIND: Chips = 2;
const S_BLIND: Chips = 1;
const MAX_N_BETS: usize = 3;

// kmeans clustering parameters
const KMEANS_TURN_CLUSTER_COUNT: usize = 128;
const KMEANS_FLOP_CLUSTER_COUNT: usize = 128;
const KMEANS_TURN_TRAINING_ITERATIONS: usize = 128;
const KMEANS_FLOP_TRAINING_ITERATIONS: usize = 128;

// mccfr parameters
const CFR_BATCH_SIZE: usize = 256;
const CFR_TREE_COUNT: usize = 67_108_864;
const CFR_ITERATIONS: usize = CFR_TREE_COUNT / CFR_BATCH_SIZE;
const CFR_DISCOUNT_PHASE: usize = 100_000;
const CFR_PRUNNING_PHASE: usize = 100_000_000;

// regret matching parameters
const REGRET_MIN: Utility = -3e5;
const REGRET_MAX: Utility = Utility::MAX;
const POLICY_MIN: Probability = Probability::MIN_POSITIVE;
fn progress(n: usize) -> indicatif::ProgressBar {
    let progress = indicatif::ProgressBar::new(n as u64);
    let style = indicatif::ProgressStyle::with_template("{pos}")
        .unwrap()
        .progress_chars("..");
    progress.set_style(style);
    progress
}
