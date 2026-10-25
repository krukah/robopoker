//! Cross-fingerprint training scoreboard.
//!
//! Reads the shared `benchmark` (external-opponent bb/100) and `litmus` (structural pass/fail)
//! tables and prints every `(regime, version, opponent, variant)` fingerprint ranked by bb/100
//! — the "which config do I commit to" view. `--json` emits the same ranking as machine-
//! readable output for tooling / CI.
mod cli;
use cli::*;

use clap::Parser;
use daybook::Scoreboard;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _telemetry = vitals::TelemetryGuard::init();
    let standings = daybook::Pool::db().await.leaderboard().await;
    if cli.json() {
        println!("{}", serde_json::to_string_pretty(&standings)?);
    } else if standings.is_empty() {
        println!("no benchmark or litmus results recorded yet");
    } else {
        println!("{}", daybook::Standing::render(&standings));
    }
    Ok(())
}
