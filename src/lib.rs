pub mod cards;
pub use cards::*;
pub mod gameplay;
pub use gameplay::*;
pub mod transport;
pub use transport::*;
pub mod wasm;
pub use wasm::*;

#[cfg(feature = "native")]
pub mod analysis;
#[cfg(feature = "native")]
pub mod clustering;
#[cfg(feature = "native")]
pub mod mccfr;
#[cfg(feature = "native")]
pub mod players;
#[cfg(feature = "native")]
pub mod save;
#[cfg(feature = "native")]
pub mod search;

/// dimensional analysis types
type Chips = i16;
type Equity = f32;
type Energy = f32;
type Entropy = f32;
type Utility = f32;
type Probability = f32;

#[cfg(feature = "native")]
static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// game tree parameters
const N: usize = 2;
const STACK: Chips = 100;
const B_BLIND: Chips = 2;
const S_BLIND: Chips = 1;
const MAX_RAISE_REPEATS: usize = 3;
const MAX_DEPTH_SUBGAME: usize = 16;

/// sinkhorn optimal transport parameters
const SINKHORN_TEMPERATURE: Entropy = 0.025;
const SINKHORN_ITERATIONS: usize = 128;
const SINKHORN_TOLERANCE: Energy = 0.001;

// kmeans clustering parameters
const KMEANS_FLOP_TRAINING_ITERATIONS: usize = 20;
const KMEANS_TURN_TRAINING_ITERATIONS: usize = 24;
const KMEANS_FLOP_CLUSTER_COUNT: usize = 128;
const KMEANS_TURN_CLUSTER_COUNT: usize = 144;
const KMEANS_EQTY_CLUSTER_COUNT: usize = 101;

/// rps mccfr parameteres
const ASYMMETRIC_UTILITY: f32 = 2.0;
const CFR_BATCH_SIZE_RPS: usize = 1;
const CFR_TREE_COUNT_RPS: usize = 8192;

// nlhe mccfr parameters
const CFR_BATCH_SIZE_NLHE: usize = 128;
const CFR_TREE_COUNT_NLHE: usize = 0x10000000;

/// profile average sampling parameters
const SAMPLING_THRESHOLD: Entropy = 1.0;
const SAMPLING_ACTIVATION: Energy = 0.0;
const SAMPLING_EXPLORATION: Probability = 0.01;

// regret matching parameters, although i haven't implemented regret clamp yet
const POLICY_MIN: Probability = Probability::MIN_POSITIVE;
const REGRET_MIN: Utility = -3e5;

pub const PROGRESS_STYLE: &str = "{spinner:.cyan} {elapsed} ~ {percent:>3}% {wide_bar:.cyan}";

/// trait for random generation, mainly (strictly?) for testing
pub trait Arbitrary {
    fn random() -> Self;
}

/// progress bar
#[cfg(feature = "native")]
pub fn progress(n: usize) -> indicatif::ProgressBar {
    let tick = std::time::Duration::from_secs(60);
    let style = indicatif::ProgressStyle::with_template(PROGRESS_STYLE).unwrap();
    let progress = indicatif::ProgressBar::new(n as u64);
    progress.set_style(style);
    progress.enable_steady_tick(tick);
    progress
}

/// initialize logging and setup graceful interrupt listener
#[cfg(feature = "native")]
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

/// get a database connection and return the client
#[cfg(feature = "native")]
pub async fn db() -> std::sync::Arc<tokio_postgres::Client> {
    log::info!("connecting to database");
    let tls = tokio_postgres::tls::NoTls;
    let ref url = std::env::var("DB_URL").expect("DB_URL must be set");
    let (client, connection) = tokio_postgres::connect(url, tls)
        .await
        .expect("database connection failed");
    tokio::spawn(connection);
    std::sync::Arc::new(client)
}

#[cfg(feature = "native")]
pub fn kys() {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!();
        log::warn!("violent interrupt received, exiting immediately");
        std::process::exit(0);
    });
}

#[cfg(feature = "native")]
pub fn brb() {
    std::thread::spawn(|| {
        let ref mut buffer = String::new();
        loop {
            buffer.clear();
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
