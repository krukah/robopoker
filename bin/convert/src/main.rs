//! Interactive CLI for Poker Analysis
//!
//! Provides an interactive command-line interface for:
//! - Type conversions (Path, Edge, Abstraction, Observation, Isomorphism)
//! - Database queries (equity, distance, population, similarity, etc.)

#[tokio::main]
async fn main() {
    let _telemetry = rbp_telemetry::init();
    rbp_core::kys();
    rbp_core::brb();
    rbp_server::CLI::run().await;
}
