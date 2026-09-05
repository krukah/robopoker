//! Evaluate the blueprint against published poker hand histories.
//!
//! ```bash
//! cargo run -p phh-cli -- conform            # replay every hand, check settlement + engine structure
//! cargo run -p phh-cli -- calibrate          # observed bet sizes vs. our Size grid
//! cargo run -p phh-cli -- calibrate --player Pluribus
//! cargo run -p phh-cli -- agree --player Pluribus    # blueprint vs. what was played
//! cargo run -p phh-cli -- duel  --player Pluribus    # what their decisions were worth
//! ```
//!
//! `agree` measures similarity and needs only a backend URL. `duel` measures
//! value, and prefers the database because it makes far too many policy lookups
//! to want a round trip in front of each one — `--api` trades that for needing
//! no credentials, by memoizing the lookups and fanning out across spots.
//!
//! Plan of record and the reasoning behind each mode:
//! `docs/active/phh-evaluation.md`.

mod agree;
mod calibrate;
mod conform;
mod duel;
mod mind;
mod remote;
mod render;

use clap::Parser;
use clap::Subcommand;

/// Default corpus: the 10,000 hands published with Brown & Sandholm (2019),
/// as vendored by the `phh-dataset` repo next to this one.
const DATA: &str = "../phh-dataset/data/pluribus";

#[derive(Parser, Debug)]
#[command(name = "phh-cli", about = "Evaluate the blueprint against published hand histories.")]
struct Cli {
    /// Directory of `.phh` files, searched recursively.
    #[arg(long, default_value = DATA, env = "PHH_DATA", global = true)]
    data: std::path::PathBuf,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Replay every hand: does settlement reproduce the log, and does the
    /// engine agree about turn order and street boundaries?
    Conform,
    /// Measure observed bet sizes against the live `Size` grid.
    Calibrate {
        /// Restrict to one player's raises, e.g. `Pluribus`.
        #[arg(long)]
        player: Option<String>,
    },
    /// Score our blueprint's postflop choices against what was actually played,
    /// on the hands that reduce to our heads-up tree.
    Agree {
        /// Restrict to one player's decisions, e.g. `Pluribus`.
        #[arg(long)]
        player: Option<String>,
        /// Invert `--player`: score everyone else instead. The control — if the
        /// blueprint agrees with the table as much as with Pluribus, agreement
        /// is measuring generic poker and not Pluribus in particular.
        #[arg(long)]
        others: bool,
        /// Backend holding the blueprint to score.
        #[arg(long, default_value = "https://robopoker.io", env = "PHH_API")]
        api: String,
    },
    /// Score what the logged decisions were *worth* against our blueprint, by
    /// playing each hand out behind both their action and ours.
    Duel(duel::Args),
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    eprintln!("reading {} ...", cli.data.display());
    let records = phh::load_dir(&cli.data)?;
    eprintln!("parsed {} hands\n", records.len());
    match cli.mode {
        Mode::Conform => conform::run(&records),
        Mode::Calibrate { player } => calibrate::run(&records, player.as_deref()),
        Mode::Agree { player, others, api } => agree::run(&records, &api, player.as_deref(), others).await,
        Mode::Duel(ref args) => duel::run(&records, args).await,
    }
}
