//! Slumbot Benchmark Binary
//!
//! Spawns concurrent benchmarks against the Slumbot API from a single
//! process. See `rbp_slumbot::Runtime` for the config surface.
//!
//! Variants: --variants a,b,c [--hands N] [--continuous] [--throttle N] [--sessions N]
//! Per-variant session override: append `*N` to any variant token to set its
//! concurrent session count, overriding `--sessions` for that variant only.
//! e.g. `--variants base*1,dirac*1,depth+dirac*4,depth+world*4`.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "slumbot")]
struct Cli {
    #[arg(long, default_value = "")]
    variants: String,
    #[arg(long, default_value_t = 1000)]
    hands: usize,
    #[arg(long)]
    continuous: bool,
    #[arg(long, default_value_t = 3)]
    throttle: usize,
    #[arg(long, default_value_t = 1)]
    sessions: usize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    rbp_slumbot::Runtime::new(
        &cli.variants,
        cli.hands,
        cli.continuous,
        cli.throttle,
        cli.sessions,
    )
    .run()
    .await;
}
