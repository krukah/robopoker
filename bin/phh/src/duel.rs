//! Tier 3 report: what the logged decisions were worth against our blueprint.
//!
//! Unlike `agree`, this reads the blueprint out of the database and into memory
//! rather than asking a backend over HTTP: a duel makes on the order of a
//! million policy lookups, and none of them can afford a round trip.
//!
//! Read `delta` against `t`. A per-decision gain inside two standard errors of
//! zero is noise however large it looks, and the comparison is built to favor us
//! (see [`phh::Ledger`]), so a positive `delta` that clears the bar is the only
//! result that means much.

use crate::mind::mind;
use crate::render::Table;
use crate::render::commas;
use crate::render::pct;
use phh::Ledger;
use phh::Record;
use pokerkit::Variant;

/// Runouts sampled per decision when the log left the board short.
///
/// The corpus deals every hole card face-up and records however much of the
/// board was run out, so this covers only the streets a hand never reached — and
/// both branches of a duel walk the same worlds, so most of the sampling noise
/// differences out before it reaches the mean.
const WORLDS: usize = 32;

/// Fixed by default so two runs of the same blueprint produce the same number.
const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Cell of the bot zoo to score: `base`, `depth`, `world`, `dirac`, or a
    /// `+`-joined combination in that order.
    #[arg(long, default_value = "base")]
    pub variant: String,

    /// Restrict to one player's decisions, e.g. `Pluribus`.
    #[arg(long)]
    pub player: Option<String>,

    /// Invert `--player`: score everyone else instead. The control.
    #[arg(long)]
    pub others: bool,

    /// Runouts per decision where the log left the board short.
    #[arg(long, default_value_t = WORLDS)]
    pub worlds: usize,

    /// Seed for the runout sampling and the rollout's own choices.
    #[arg(long, default_value_t = SEED)]
    pub seed: u64,

    /// Consult the subgame solver at every rollout decision instead of the
    /// blueprint. Seconds per lookup — pair it with a small `--limit`.
    #[arg(long)]
    pub solve: bool,

    /// Score only the first N decisions.
    #[arg(long)]
    pub limit: Option<usize>,

    /// Score a running backend over HTTP instead of hydrating from `$DB_URL`.
    /// Slower per lookup but needs no credentials — see [`crate::remote`]. The
    /// deployment serves whichever blueprint it was started with, so `--variant`
    /// here selects the *solver* route rather than a local brain, and `--solve`
    /// is implied by any variant that has one.
    #[arg(long)]
    pub api: Option<String>,
}

pub async fn run(records: &[Record], args: &Args) -> anyhow::Result<()> {
    let spots = phh::harvest(records, args.player.as_deref(), args.others);
    let spots = &spots[..args.limit.unwrap_or(spots.len()).min(spots.len())];
    eprintln!("rolling out {} spots × {} worlds ...", commas(spots.len()), args.worlds);
    let ledger = match args.api.as_deref() {
        Some(api) => remote(api, spots, args)?,
        None => local(spots, args).await?,
    };
    report(&ledger, args, spots.len());
    Ok(())
}

/// The in-memory path: hydrate the blueprint once, then roll out serially. Every
/// lookup is a map read, so there is nothing for concurrency to hide.
async fn local(spots: &[phh::Spot], args: &Args) -> anyhow::Result<Ledger> {
    let variant = Variant::parse(&args.variant).ok_or_else(|| anyhow::anyhow!("unknown variant `{}`", args.variant))?;
    eprintln!("hydrating the blueprint ...");
    let model = parlor::hydrate_blueprint(daybook::db().await).await;
    let oracle = mind(variant, model, args.solve).ok_or_else(|| anyhow::anyhow!("`fish` has no blueprint to score"))?;
    Ok(spots
        .iter()
        .filter_map(|spot| phh::duel(&oracle, spot, &phh::worlds(spot, args.worlds, args.seed), args.seed))
        .collect())
}

/// The over-the-wire path: memoize and fan out across spots, because the wall
/// clock is round trips.
///
/// `--variant` picks the route. `base` is a blueprint lookup; every other cell
/// runs a CFR re-solve on the backend at roughly a second apiece, so pair those
/// with a small `--limit` unless you mean to occupy the deployment for hours.
fn remote(api: &str, spots: &[phh::Spot], args: &Args) -> anyhow::Result<Ledger> {
    let variant = Variant::parse(&args.variant).ok_or_else(|| anyhow::anyhow!("unknown variant `{}`", args.variant))?;
    let route = crate::remote::endpoint(variant).ok_or_else(|| anyhow::anyhow!("`fish` has no blueprint to score"))?;
    let oracle = crate::remote::Remote::new(api, route);
    eprintln!("route /strategy/{route}{}", if oracle.solves() { " — a CFR re-solve per lookup" } else { "" });
    let ledger = crate::remote::ledger(&oracle, spots, args.worlds, args.seed);
    eprintln!("{} distinct information sets queried", commas(oracle.hits()));
    Ok(ledger)
}

fn report(all: &Ledger, args: &Args, attempted: usize) {
    let who = match (args.player.as_deref(), args.others) {
        (Some(name), false) => name.to_string(),
        (Some(name), true) => format!("everyone but {name}"),
        (None, _) => "every player".to_string(),
    };
    println!(
        "## Decision value — {who} vs `{}` ({} of {} spots scored, {} rollouts)\n",
        args.variant,
        commas(all.n()),
        commas(attempted),
        commas(all.worlds()),
    );
    let mut table = Table::new(&["street", "n", "delta", "per 100", "stderr", "t"], &[8, 8, 10, 10, 10, 8]);
    let row = |label: &str, l: &Ledger| {
        vec![
            label.to_string(),
            commas(l.n()),
            format!("{:+.4}", l.delta()),
            format!("{:+.2}", l.per_hundred()),
            format!("{:.4}", l.stderr()),
            format!("{:+.2}", l.t()),
        ]
    };
    for street in [deuce::Street::Flop, deuce::Street::Turn, deuce::Street::Rive] {
        table.push(row(&format!("{street:?}"), &all.on(street)));
    }
    table.push(row("all", all));
    print!("{table}");
    println!("\n### What each logged action was worth\n");
    let mut table = Table::new(&["they played", "n", "share", "delta"], &[12, 8, 8, 10]);
    for (edge, n, delta) in all.by_edge() {
        table.push(vec![format!("{edge}"), commas(n), pct(n, all.n()), format!("{delta:+.4}")]);
    }
    print!("{table}");
    println!(
        "\n`delta` is big blinds the logged decision gained over ours, playing both out with our\n\
         own strategy on both seats in the same cards. Positive means they out-decided us. The\n\
         comparison leans our way — their action was chosen against human professionals, not\n\
         against us, and our bot plays the continuation — so a positive `delta` is real evidence\n\
         and a negative one is not. `t` is `delta` in standard errors; inside ±2 the sign is\n\
         noise. Blueprint had nothing to say at {} rollout decisions.",
        commas(all.misses()),
    );
}
