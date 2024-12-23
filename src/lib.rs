use cards::street::Street;

pub mod analysis;
pub mod cards;
pub mod clustering;
pub mod gameplay;
pub mod mccfr;
pub mod players;
pub mod search;
pub mod transport;

/// dimensional analysis types
type Chips = i16;
type Equity = f32;
type Energy = f32;
type Entropy = f32;
type Utility = f32;
type Probability = f32;

// game tree parameters
const N: usize = 2;
const STACK: Chips = 100;
const B_BLIND: Chips = 2;
const S_BLIND: Chips = 1;
const MAX_N_BETS: usize = 3;

/// sinkhorn optimal transport parameters
const SINKHORN_TEMPERATURE: Entropy = 0.125;
const SINKHORN_ITERATIONS: usize = 16;
const SINKHORN_TOLERANCE: Energy = 0.001;

// kmeans clustering parameters
const KMEANS_TURN_TRAINING_ITERATIONS: usize = 32;
const KMEANS_FLOP_TRAINING_ITERATIONS: usize = 32;
const KMEANS_TURN_CLUSTER_COUNT: usize = 16;
const KMEANS_FLOP_CLUSTER_COUNT: usize = 16;
const KMEANS_EQTY_CLUSTER_COUNT: usize = 64;

// mccfr parameters
const CFR_BATCH_SIZE: usize = 256;
const CFR_TREE_COUNT: usize = 1_048_576;
const CFR_ITERATIONS: usize = CFR_TREE_COUNT / CFR_BATCH_SIZE;
const CFR_PRUNNING_PHASE: usize = 100_000_000 / CFR_BATCH_SIZE;
const CFR_DISCOUNT_PHASE: usize = 100_000 / CFR_BATCH_SIZE;

// regret matching parameters
const REGRET_MIN: Utility = -3e5;
const REGRET_MAX: Utility = Utility::MAX;
const POLICY_MIN: Probability = Probability::MIN_POSITIVE;

/// street-level properties that can be written to and read from disk,
/// may or may not be dependent on other entities being written/in memory.
/// or in the case of River Abstractions, we can just generate it from scratch
/// on the fly if we need to.
pub trait Save: Sized {
    fn save(&self);
    fn done(street: Street) -> bool;
    fn load(street: Street) -> Self;
    fn make(street: Street) -> Self;
    fn push(street: Street) -> Self {
        if Self::done(street) {
            log::info!(
                "loading {} from file {street}",
                std::any::type_name::<Self>()
            );
            Self::load(street)
        } else {
            log::info!(
                "writing {} into file {street}",
                std::any::type_name::<Self>()
            );
            Self::make(street)
        }
    }
}

/// trait for random generation, mainly (strictly?) for testing
pub trait Arbitrary {
    fn random() -> Self;
}

/// progress bar
pub fn progress(n: usize) -> indicatif::ProgressBar {
    let tick = std::time::Duration::from_secs(1);
    let style = "{spinner:.cyan} {elapsed} ~ {percent:>3}% {wide_bar:.cyan}";
    let style = indicatif::ProgressStyle::with_template(style).unwrap();
    let progress = indicatif::ProgressBar::new(n as u64);
    progress.set_style(style);
    progress.enable_steady_tick(tick);
    progress
}

/// initialize logging
pub fn logs() {
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
