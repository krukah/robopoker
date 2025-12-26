//! Analysis Server Binary
//!
//! Runs the HTTP analysis server for querying training results.

use robopoker::*;

#[tokio::main]
async fn main() {
    log();
    kys();
    brb();
    analysis::Server::run().await.unwrap();
}
