//! hosting Server Binary
//!
//! Runs the HTTP server for hosting live game rooms.
//! Supports WebSocket connections for real-time play.

use robopoker::*;

#[tokio::main]
async fn main() {
    log();
    kys();
    brb();
    hosting::Server::run().await.unwrap();
}
