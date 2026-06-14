//! Interactive CLI for Poker Analysis
//!
//! Provides an interactive command-line interface for:
//! - Type conversions (Path, Edge, Abstraction, Observation, Isomorphism)
//! - Database queries (equity, distance, population, similarity, etc.)

#[tokio::main]
async fn main() {
    let _telemetry = vitals::init();
    fulcrum::kys();
    fulcrum::brb();
    portal::CLI::run().await;
}
