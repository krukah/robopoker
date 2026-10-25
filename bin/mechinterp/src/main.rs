//! Mechanistic interpretability pass over the learned abstraction.
//!
//! Samples every bucket on the requested streets, runs each sampled member through the
//! [`rosetta`] feature vocabulary, and upserts a name and a description per bucket into
//! `mechinterp`.
mod cli;
mod scope;
use cli::*;
use scope::*;

use clap::Parser;
use clap::ValueEnum;
use deuce::Street;
use rosetta::*;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _telemetry = vitals::TelemetryGuard::init();
    let db = daybook::Pool::db().await;
    for street in cli.streets() {
        let glossary = Glossary::from(db.profiles(street, cli.samples()).await);
        eprintln!("{street}: {} buckets", glossary.len());
        if cli.long() {
            glossary.glosses().iter().for_each(|gloss| println!("{gloss}\n"));
        } else {
            print!("{glossary}");
        }
        if !cli.dry() {
            db.inscribe(&glossary).await;
        }
    }
    Ok(())
}
