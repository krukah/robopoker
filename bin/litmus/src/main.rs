//! Thin CLI wrapper around `litmus::Litmus`.
//!
//! Parses CLI args, wires strategy/training APIs, runs the catalog,
//! writes a markdown report.

mod cli;
mod universe;
use cli::*;
use universe::*;

use clap::Parser;
use daybook::Ensure;
use daybook::Scoreboard;
use std::path::PathBuf;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.universe().boot();

    let scenarios = litmus::Schema::load(cli.scenarios())?;
    let client = daybook::Pool::db().await;
    let backend = portal::litmus::Backend::new(
        portal::StrategyAPI::new(client.clone()),
        portal::TrainingAPI::new(client.clone()),
    );
    let litmus = litmus::Litmus::new(backend);

    let outcomes = litmus.run(&scenarios).await?;

    let (mut pass, mut fail, mut skip, mut error) = (0, 0, 0, 0);
    for o in &outcomes {
        match o.status() {
            litmus::Status::Pass => pass += 1,
            litmus::Status::Fail => fail += 1,
            litmus::Status::Skip => skip += 1,
            litmus::Status::Error => error += 1,
        }
        if !matches!(o.status(), litmus::Status::Pass) {
            eprintln!("  [{}] {}: {}", o.status().label(), o.case().name(), o.detail());
        }
    }

    let api_label = format!("rbp-{} {}", pokerkit::Regime::current(), pokerkit::Version::current());
    let status = litmus.status().await.ok();
    let grid_usage = litmus.grid_usage().await.ok();
    let report = litmus::Render::render(&api_label, status.as_ref(), &scenarios, &outcomes, grid_usage.as_deref());

    if let Some(path) = cli.out() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &report)?;
        eprintln!("wrote {}", path.display());
    } else {
        print!("{report}");
    }

    let stamped = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time")
        .as_secs() as i64;
    let epoch = status.as_ref().map_or(0, kicker::ApiStatus::epoch);
    client.ensure::<daybook::Litmus>().await;
    client
        .record_litmus(epoch, pass as i64, fail as i64, skip as i64, error as i64, stamped)
        .await;

    eprintln!("summary: {pass} pass, {fail} fail, {skip} skip, {error} error");
    if fail + error > 0 {
        std::process::exit(1);
    }
    Ok(())
}
