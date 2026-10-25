//! Slumbot Benchmark Binary
//!
//! Spawns concurrent benchmarks against the Slumbot API from a single process. See
//! `bout::Runtime` for the config surface and `spar` for the Slumbot protocol.

mod allocator;
mod cli;
mod universe;
use cli::*;
use universe::*;

use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let telemetry = cli.universe().boot();
    bout::Runtime::<spar::Session>::new(
        (),
        cli.variants(),
        cli.hands(),
        cli.continuous(),
        cli.throttle(),
        cli.sessions(),
    )
    .run()
    .await;
    std::mem::drop(telemetry);
}
