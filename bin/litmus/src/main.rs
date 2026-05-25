//! Thin CLI wrapper around `rbp_litmus::Litmus`.
//!
//! Parses CLI args, wires strategy/training APIs, runs the catalog,
//! writes a markdown report.

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "litmus", about = "Static blueprint validation runner.")]
struct Cli {
    /// Path to scenarios.json.
    #[arg(long, default_value = "bin/litmus/scenarios.json")]
    scenarios: PathBuf,

    /// Where to write the markdown report. If omitted, prints to stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let scenarios = rbp_litmus::load(&cli.scenarios)?;
    let client = rbp_database::db().await;
    let backend = rbp_server::litmus::Backend::new(
        rbp_server::StrategyAPI::new(client.clone()),
        rbp_server::TrainingAPI::new(client),
    );
    let litmus = rbp_litmus::Litmus::new(backend);

    let outcomes = litmus.run(&scenarios).await?;

    let (mut pass, mut fail, mut skip, mut error) = (0, 0, 0, 0);
    for o in &outcomes {
        match o.status {
            rbp_litmus::Status::Pass => pass += 1,
            rbp_litmus::Status::Fail => fail += 1,
            rbp_litmus::Status::Skip => skip += 1,
            rbp_litmus::Status::Error => error += 1,
        }
        if !matches!(o.status, rbp_litmus::Status::Pass) {
            eprintln!("  [{}] {}: {}", o.status.label(), o.case.name, o.detail);
        }
    }

    let api_label = format!("rbp-{} {}", rbp_core::regime(), rbp_core::version());
    let status = litmus.status().await.ok();
    let grid_usage = litmus.grid_usage().await.ok();
    let report = rbp_litmus::render(&api_label, status.as_ref(), &scenarios, &outcomes, grid_usage.as_deref());

    if let Some(path) = &cli.out {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &report)?;
        eprintln!("wrote {}", path.display());
    } else {
        print!("{report}");
    }

    eprintln!("summary: {pass} pass, {fail} fail, {skip} skip, {error} error");
    if fail + error > 0 {
        std::process::exit(1);
    }
    Ok(())
}
