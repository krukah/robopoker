//! Unified Backend Binary
//!
//! Combines analysis API and live game hosting into a single server.
//! Runs on BIND_ADDR (e.g. 0.0.0.0:8888).

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    portal::run().await.unwrap();
}
