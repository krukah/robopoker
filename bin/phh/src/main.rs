//! Evaluate the blueprint against published poker hand histories.
//!
//! `agree` measures similarity and needs only a backend URL. `duel` measures value, and prefers
//! the database because it makes far too many policy lookups to want a round trip in front of
//! each one — `--api` trades that for needing no credentials, by memoizing the lookups and
//! fanning out across spots.

mod cli;
mod consts;
mod minds;
mod mode;
use cli::*;
use consts::*;
use mode::*;

mod agree;
mod calibrate;
mod conform;
mod duel;
mod mind;
mod remote;
mod render;

use clap::Parser;
use clap::Subcommand;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    eprintln!("reading {} ...", cli.data().display());
    let records = phh::Parse::load_dir(cli.data())?;
    eprintln!("parsed {} hands\n", records.len());
    match cli.mode() {
        Mode::Conform => conform::Conform::run(&records),
        Mode::Calibrate { player } => calibrate::Calibrate::run(&records, player.as_deref()),
        Mode::Agree { player, others, api } => agree::Agree::run(&records, api, player.as_deref(), *others).await,
        Mode::Duel(args) => duel::Args::run(&records, args).await,
    }
}
