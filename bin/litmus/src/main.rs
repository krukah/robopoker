//! Thin CLI wrapper around `litmus::Litmus`.
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

    let scenarios = litmus::load(&cli.scenarios)?;
    let client = ledger::db().await;
    let backend =
        portal::litmus::Backend::new(portal::StrategyAPI::new(client.clone()), portal::TrainingAPI::new(client));
    let litmus = litmus::Litmus::new(backend);

    let outcomes = litmus.run(&scenarios).await?;

    let (mut pass, mut fail, mut skip, mut error) = (0, 0, 0, 0);
    for o in &outcomes {
        match o.status {
            litmus::Status::Pass => pass += 1,
            litmus::Status::Fail => fail += 1,
            litmus::Status::Skip => skip += 1,
            litmus::Status::Error => error += 1,
        }
        if !matches!(o.status, litmus::Status::Pass) {
            eprintln!("  [{}] {}: {}", o.status.label(), o.case.name, o.detail);
        }
    }

    let api_label = format!("rbp-{} {}", pokerkit::regime(), pokerkit::version());
    let status = litmus.status().await.ok();
    let grid_usage = litmus.grid_usage().await.ok();
    let report = litmus::render(&api_label, status.as_ref(), &scenarios, &outcomes, grid_usage.as_deref());

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
