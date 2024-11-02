pub mod cards;
pub mod clustering;
pub mod kmeans;
pub mod mccfr;
pub mod play;
pub mod players;
pub mod rts;
pub mod transport;

type Equity = f32;
type Utility = f32;
type Probability = f32;

const KMEANS_TURN_CLUSTER_COUNT: usize = 100;
const KMEANS_FLOP_CLUSTER_COUNT: usize = 100;
const KMEANS_TURN_TRAINING_ITERATIONS: usize = 100;
const KMEANS_FLOP_TRAINING_ITERATIONS: usize = 100;

const CFR_BATCH_SIZE: usize = 16;
const CFR_TREE_COUNT: usize = 16_777_216;
const CFR_ITERATIONS: usize = CFR_TREE_COUNT / CFR_BATCH_SIZE;
const CFR_DISCOUNT_PHASE: usize = 100_000;
const CFR_PRUNNING_PHASE: usize = 100_000_000;

const REGRET_MIN: Utility = -3e5;
const REGRET_MAX: Utility = Utility::MAX;
const POLICY_MIN: Probability = Probability::MIN_POSITIVE;

const MAX_N_BETS: usize = 3;
