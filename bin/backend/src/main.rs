//! Unified Backend Binary
//!
//! Combines analysis API and live game hosting into a single server.
//! Runs on BIND_ADDR (e.g. 0.0.0.0:8888).

#[tokio::main]
async fn main() {
    rbp_core::log();
    rbp_core::kys();
    rbp_core::brb();
    rbp_server::run().await.unwrap();
}
