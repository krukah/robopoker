pub mod cards;
pub mod clustering;
pub mod gameplay;
pub mod kmeans;
pub mod mccfr;
pub mod players;
pub mod search;
pub mod transport;

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

// kmeans clustering parameters
const KMEANS_TURN_CLUSTER_COUNT: usize = 128;
const KMEANS_FLOP_CLUSTER_COUNT: usize = 128;
const KMEANS_TURN_TRAINING_ITERATIONS: usize = 128;
const KMEANS_FLOP_TRAINING_ITERATIONS: usize = 128;

// mccfr parameters
const CFR_BATCH_SIZE: usize = 9_182;
const CFR_TREE_COUNT: usize = 68_719_476_736;
const CFR_ITERATIONS: usize = CFR_TREE_COUNT / CFR_BATCH_SIZE;
const CFR_DISCOUNT_PHASE: usize = 100_000;
const CFR_PRUNNING_PHASE: usize = 100_000_000;

// regret matching parameters
const REGRET_MIN: Utility = -3e5;
const REGRET_MAX: Utility = Utility::MAX;
const POLICY_MIN: Probability = Probability::MIN_POSITIVE;

pub trait Arbitrary {
    fn random() -> Self;
}

pub fn progress(n: usize) -> indicatif::ProgressBar {
    let tick = std::time::Duration::from_secs(5);
    let style = "{percent:>2}% {spinner:.cyan} {elapsed} ETA {eta} {wide_bar:.cyan}";
    let style = indicatif::ProgressStyle::with_template(style).unwrap();
    let progress = indicatif::ProgressBar::new(n as u64);
    progress.set_style(style);
    progress.enable_steady_tick(tick);
    progress
}

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
