//! Autotrain Binary
//!
//! Unified training pipeline with postgres as source of truth.
//!
//! Options: --status, --fast, --slow, --cluster, --reset

#[tokio::main]
async fn main() {
    rbp_core::log();
    rbp_core::kys();
    rbp_core::brb();
    rbp_autotrain::Mode::run().await;
}
