//! Mechanistic interpretability pass over the learned abstraction.
//!
//! Samples every bucket on the requested streets, runs each sampled member
//! through the [`rosetta`] feature vocabulary, and upserts a name and a
//! description per bucket into `mechinterp`. Defaults to the two learned
//! streets — the flop and turn k-means buckets — since preflop buckets are
//! single hands and river buckets are equity bands, both of which already name
//! themselves.
use clap::Parser;
use clap::ValueEnum;
use deuce::Street;
use rosetta::*;

/// What a `--streets` value names. A plain `Vec<String>` matched against
/// `Street::try_from` swallowed typos: `--streets xyz` parsed to nothing and
/// the run silently interpreted no buckets at all.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum Scope {
    #[value(alias = "preflop")]
    Pref,
    Flop,
    Turn,
    #[value(alias = "river")]
    Rive,
    All,
}

impl Scope {
    fn streets(self) -> Vec<Street> {
        match self {
            Self::All => Street::all().to_vec(),
            Self::Pref => vec![Street::Pref],
            Self::Flop => vec![Street::Flop],
            Self::Turn => vec![Street::Turn],
            Self::Rive => vec![Street::Rive],
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "mechinterp",
    about = "Name and describe every abstraction bucket from a sample of its members."
)]
struct Cli {
    /// Members sampled per bucket. A hundred pins every frequency to a few points.
    #[arg(long, default_value_t = 128)]
    samples: usize,
    /// Streets to interpret: any of pref, flop, turn, rive — or all.
    #[arg(long, value_enum, value_delimiter = ',', default_value = "flop,turn")]
    streets: Vec<Scope>,
    /// Print the glossary without writing it.
    #[arg(long)]
    dry: bool,
    /// Print each description, not just each name.
    #[arg(long)]
    long: bool,
}

impl Cli {
    fn streets(&self) -> Vec<Street> {
        self.streets.iter().copied().flat_map(Scope::streets).collect()
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _telemetry = vitals::init();
    let db = daybook::db().await;
    for street in cli.streets() {
        let glossary = Glossary::from(db.profiles(street, cli.samples).await);
        eprintln!("{street}: {} buckets", glossary.len());
        if cli.long {
            glossary.glosses().iter().for_each(|gloss| println!("{gloss}\n"));
        } else {
            print!("{glossary}");
        }
        if !cli.dry {
            db.inscribe(&glossary).await;
        }
    }
    Ok(())
}
