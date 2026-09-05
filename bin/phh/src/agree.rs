//! Tier 2 report: our blueprint's choices against Pluribus's.
//!
//! Asks a running backend rather than the database, so this needs no
//! credentials — `--api` points at whichever deployment holds the blueprint you
//! want scored.
//!
//! Read the numbers against their floors. `naive` is what you score by always
//! guessing the single most common action; `spread` is how many edges were on
//! offer. Agreement below `naive` or perplexity near `spread` means the
//! blueprint told us nothing at those spots.

use crate::render::Table;
use crate::render::commas;
use crate::render::pct;
use futures::StreamExt;
use kicker::Edge;
use kicker::dto::GetPolicy;
use phh::Agreement;
use phh::Policy;
use phh::Record;
use phh::Spot;
use phh::Verdict;
use pokerkit::Probability;

/// Concurrent policy queries. The backend answers in ~2ms; the wall clock is
/// round trips, and this keeps 3.5k of them to about a minute without leaning
/// on a shared deployment.
const LANES: usize = 16;

pub async fn run(records: &[Record], api: &str, player: Option<&str>, others: bool) -> anyhow::Result<()> {
    let spots = phh::harvest(records, player, others);
    eprintln!("querying {} spots against {api} ...", commas(spots.len()));
    let client = reqwest::Client::builder().build()?;
    let verdicts = futures::stream::iter(spots.iter().map(|spot| ask(&client, api, spot)))
        .buffer_unordered(LANES)
        .collect::<Vec<_>>()
        .await;
    let failed = verdicts.iter().filter(|v| v.is_err()).count();
    let agreement = verdicts.into_iter().flatten().flatten().collect::<Agreement>();
    report(&agreement, player, others, failed);
    Ok(())
}

/// One policy query, scored.
async fn ask(client: &reqwest::Client, api: &str, spot: &Spot) -> anyhow::Result<Option<Verdict>> {
    let policy = client
        .post(format!("{api}/strategy/policy"))
        .json(&GetPolicy::from(spot.witness()))
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    let weights = policy
        .get("policy")
        .and_then(|p| p.get("accumulated"))
        .and_then(serde_json::Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((Edge::try_from(k.as_str()).ok()?, v.as_f64()? as Probability)))
                .collect::<Policy>()
        })
        .unwrap_or_default();
    Ok(Verdict::new(spot, &weights))
}

fn report(scored: &Agreement, player: Option<&str>, others: bool, failed: usize) {
    let who = match (player, others) {
        (Some(name), false) => name.to_string(),
        (Some(name), true) => format!("everyone but {name}"),
        (None, _) => "every player".to_string(),
    };
    println!("## Policy agreement — {who} ({} spots scored, {failed} failed)\n", commas(scored.n()));
    let mut table = Table::new(
        &[
            "street",
            "n",
            "agree",
            "naive",
            "lift",
            "perplexity",
            "spread",
            "no mass",
        ],
        &[8, 7, 8, 8, 8, 12, 8, 9],
    );
    let row = |label: &str, a: &Agreement| {
        vec![
            label.to_string(),
            commas(a.n()),
            format!("{:.1}%", 100.0 * a.rate()),
            format!("{:.1}%", 100.0 * a.naive()),
            format!("{:+.1}pp", 100.0 * (a.rate() - a.naive())),
            format!("{:.2}", a.perplexity()),
            format!("{:.2}", a.spread()),
            pct(a.impossible(), a.n()),
        ]
    };
    for street in [deuce::Street::Flop, deuce::Street::Turn, deuce::Street::Rive] {
        table.push(row(&format!("{street:?}"), &scored.on(street)));
    }
    table.push(row("all", scored));
    print!("{table}");
    println!("\n### Where the disagreements are\n");
    let mut table = Table::new(&["pluribus played", "we chose", "n", "share"], &[16, 10, 7, 8]);
    for (played, chosen, n) in scored.confusion().iter().take(12) {
        table.push(vec![
            format!("{played}"),
            format!("{chosen}"),
            commas(*n),
            pct(*n, scored.n()),
        ]);
    }
    print!("{table}");
    println!("\n### Action mix — how often each edge is played vs chosen\n");
    let mut table = Table::new(&["edge", "pluribus", "us", "delta"], &[8, 10, 10, 10]);
    for (edge, played, chosen) in scored.bias() {
        table.push(vec![
            format!("{edge}"),
            format!("{:.1}%", 100.0 * played),
            format!("{:.1}%", 100.0 * chosen),
            format!("{:+.1}pp", 100.0 * (chosen - played)),
        ]);
    }
    print!("{table}");
    println!(
        "\nAgreement with Pluribus is not correctness — it played a different game and is not\n\
         an equilibrium. `naive` is the score from always guessing the most common action and\n\
         `spread` is the mean number of edges on offer; a row that fails to beat its floors is\n\
         a row where the blueprint said nothing. For what the disagreements actually cost,\n\
         which is a different question, run `duel`."
    );
}
