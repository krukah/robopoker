//! Interactive CLI for Poker Analysis
//!
//! Provides an interactive command-line interface for:
//! - Type conversions (Path, Edge, Abstraction, Observation, Isomorphism)
//! - Database queries (equity, distance, population, similarity, etc.)

use robopoker::*;

#[tokio::main]
async fn main() {
    log();
    kys();
    brb();
    analysis::CLI::run().await;
}
