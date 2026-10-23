//! Training pipeline. Mode (exactly one of): --status, --fast, --slow, --cluster, --recluster,
//! --reset, --forget, --deep [DIR].
//!
//! --recluster drops the clustering tables, as --reset drops the blueprint; the next
//! --fast/--slow/--cluster rebuilds them. --deep trains Deep CFR on NLHE with no database.

mod allocator;
mod cli;
mod universe;
use cli::*;
use universe::*;

use clap::ArgGroup;
use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    cli.universe().boot();
    cli.mode().run().await;
}
