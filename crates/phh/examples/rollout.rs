//! Smoke-test the Tier 3 rollout against the real corpus, with no database.
//!
//! [`Ledger`] normally scores a hydrated blueprint, which needs credentials the plumbing itself
//! does not.

use kicker::Edge;
use kicker::Recall;
use kicker::Witness;
use phh::Ledger;
use phh::Oracle;
use phh::Policy;

/// Checks when it can and calls when it cannot. Enough to drive a rollout to a
/// terminal node without a blueprint.
struct Passive;

impl Oracle for Passive {
    fn policy(&self, witness: &Witness) -> Policy {
        [(if witness.head().may_check() { Edge::Check } else { Edge::Call }, 1.0)]
            .into_iter()
            .collect()
    }
}

fn main() -> anyhow::Result<()> {
    let data = std::env::var("PHH_DATA").unwrap_or_else(|_| "../phh-dataset/data/pluribus".to_string());
    let records = phh::Parse::load_dir(std::path::Path::new(&data))?;
    let spots = phh::Spot::harvest(&records, Some("Pluribus"), false);
    println!("{} hands, {} spots", records.len(), spots.len());
    let ledger = spots
        .iter()
        .filter_map(|spot| phh::Duel::play(&Passive, spot, &phh::Duel::worlds(spot, 8, 0), 0))
        .collect::<Ledger>();
    println!(
        "scored {} of {} spots over {} rollouts, {} misses",
        ledger.n(),
        spots.len(),
        ledger.worlds(),
        ledger.misses(),
    );
    println!("delta {:+.4} bb ± {:.4} (t {:+.2})", ledger.delta(), ledger.stderr(), ledger.t());
    for (edge, n, delta) in ledger.by_edge() {
        println!("  {edge:<8} n={n:<6} delta {delta:+.4}");
    }
    anyhow::ensure!(ledger.n() > 0, "no spot in the corpus produced a rollout");
    anyhow::ensure!(ledger.misses() == 0, "a passive strategy has an opinion everywhere");
    Ok(())
}
