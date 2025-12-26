//! Autotrain Binary
//!
//! Unified training pipeline with postgres as source of truth.
//!
//! Options: --status, --fast, --slow, --cluster

use robopoker::*;

#[tokio::main]
async fn main() {
    log();
    kys();
    brb();
    autotrain::Mode::run().await;
}
