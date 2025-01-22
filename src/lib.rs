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
const N_RAISE: usize = 3;

/// sinkhorn optimal transport parameters
const SINKHORN_TEMPERATURE: Entropy = 0.005;
const SINKHORN_ITERATIONS: usize = 1024;
const SINKHORN_TOLERANCE: Energy = 0.01;

// kmeans clustering parameters
const KMEANS_FLOP_TRAINING_ITERATIONS: usize = 32; // eyeball test seems to converge around here for K = 128
const KMEANS_TURN_TRAINING_ITERATIONS: usize = 32; // eyeball test seems to converge around here for K = 144
const KMEANS_FLOP_CLUSTER_COUNT: usize = 128;
const KMEANS_TURN_CLUSTER_COUNT: usize = 144;
const KMEANS_EQTY_CLUSTER_COUNT: usize = 101;

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

/// trait for random generation, mainly (strictly?) for testing
pub trait Arbitrary {
    fn random() -> Self;
}

/// street-level properties that can be written to and read from disk,
/// may or may not be dependent on other entities being written/in memory.
/// or in the case of River Abstractions, we can just generate it from scratch
/// on the fly if we need to.
pub trait Save: Sized {
    fn name() -> &'static str;
    fn save(&self);
    fn load(street: Street) -> Self;
    fn make(street: Street) -> Self;
    fn done(street: Street) -> bool {
        std::fs::metadata(Self::path(street)).is_ok()
    }
    fn path(street: Street) -> String {
        format!("{}{}", Self::name(), street)
    }
}

/// progress bar
pub fn progress(n: usize) -> indicatif::ProgressBar {
    let tick = std::time::Duration::from_secs(60);
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

pub async fn db() -> std::sync::Arc<tokio_postgres::Client> {
    log::info!("connecting to database");
    let tls = tokio_postgres::tls::NoTls;
    let ref url = std::env::var("DATABASE_URL").expect("set database url in environment");
    let (client, connection) = tokio_postgres::connect(url, tls)
        .await
        .expect("database connection failed");
    tokio::spawn(connection);
    std::sync::Arc::new(client)
}
