//! Unified Backend Binary
//!
//! Combines topology API and live game hosting into a single server.
//! Runs on BIND_ADDR (e.g. 0.0.0.0:8888).
//!
//! Universe: --regime pluribus|slumbot --version v1

mod allocator;
mod cli;
mod universe;
use cli::*;
use universe::*;

use clap::Parser;

#[tokio::main]
async fn main() {
    Cli::parse().universe().boot();
    portal::Server::run().await.unwrap();
}
