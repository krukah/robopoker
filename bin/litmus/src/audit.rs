//! Static audit of scenarios.json against the current action grid.
//!
//! the test will silently see 0% regardless of policy.
//!
//! the current grid) → the test cannot run.
//!
//! No database access required. Pure replay over `kicker`.

#[path = "audit/cli.rs"]
mod cli;
#[path = "audit/row.rs"]
mod row;
mod universe;
#[path = "audit/verdict.rs"]
mod verdict;
use clap::Parser;
use cli::*;
use kicker::*;
use litmus::*;
use row::*;
use std::collections::BTreeMap;
use universe::*;
use verdict::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.universe().boot();
    let scenarios = Schema::load(cli.scenarios())?;
    let catalog = Catalog::new(&scenarios);
    let cases = Compose::resolve(&scenarios)?;

    let mut rows = Vec::new();
    for case in &cases {
        let hands = case
            .hand()
            .map(|h| vec![h.to_string()])
            .or_else(|| case.hands().cloned())
            .unwrap_or_default();
        let histories = if case.history().is_empty() {
            case.histories().cloned().unwrap_or_default()
        } else {
            vec![case.history().to_string()]
        };
        for hand in &hands {
            for history in &histories {
                rows.push(Audit::audit_one(&catalog, case.name(), hand, history, case.edge()));
            }
        }
    }

    let mut by_verdict: BTreeMap<&'static str, usize> = BTreeMap::new();
    for row in &rows {
        let key = match row.verdict() {
            Verdict::OnGrid => "on-grid",
            Verdict::OffGrid => "off-grid",
            Verdict::OffTree(_) => "off-tree",
        };
        *by_verdict.entry(key).or_default() += 1;
    }

    println!("# Litmus scenario audit (action grid)\n");
    println!("**API**: `rbp-{} {}`\n", pokerkit::Regime::current(), pokerkit::Version::current());
    println!("## Summary\n");
    for (k, n) in &by_verdict {
        println!("- **{k}**: {n}");
    }
    println!("- **total queries**: {}\n", rows.len());

    for verdict_label in ["off-tree", "off-grid", "on-grid"] {
        let filtered = rows
            .iter()
            .filter(|r| match (&r.verdict(), verdict_label) {
                (Verdict::OffTree(_), "off-tree") => true,
                (Verdict::OffGrid, "off-grid") => true,
                (Verdict::OnGrid, "on-grid") => true,
                _ => false,
            })
            .collect::<Vec<_>>();
        if filtered.is_empty() {
            continue;
        }
        println!("## {verdict_label} ({})\n", filtered.len());
        println!("| case | hand | history | spr | queried | legal | detail |");
        println!("|---|---|---|---|---|---|---|");
        for r in filtered {
            let detail = match &r.verdict() {
                Verdict::OffTree(e) => e.replace('|', "/"),
                _ => String::new(),
            };
            println!(
                "| {} | {} | {} | {} | `{}` | {} | {} |",
                r.case(),
                r.hand(),
                r.history(),
                r.spr(),
                r.queried(),
                r.legal(),
                detail
            );
        }
        println!();
    }

    Ok(())
}

/// Namespace for the audit routines.
pub(crate) struct Audit;

impl Audit {
    fn audit_one(catalog: &Catalog, case: &str, hand: &str, history: &str, edge: &str) -> Row {
        match Catalog::build_witness(catalog, hand, history) {
            Err(e) => Row::new(
                case.to_string(),
                hand.to_string(),
                history.to_string(),
                edge.to_string(),
                "—".to_string(),
                "—".to_string(),
                Verdict::OffTree(format!("{e}")),
            ),
            Ok(witness) => {
                let spr = format!("{:?}", witness.head().geometry());
                let choices = witness.choices().collect::<Vec<Edge>>();
                let legal = choices.iter().map(|e| format!("`{e}`")).collect::<Vec<_>>().join(" ");
                let verdict = match Catalog::parse_edge(edge) {
                    Err(_) => {
                        // Wildcard edges like "*" used by `single` tests aren't typed
                        // Edges — treat as on-grid (the runner sums over all choices).
                        Verdict::OnGrid
                    }
                    Ok(target) => {
                        if choices.iter().any(|c| Audit::edge_matches(c, &target)) {
                            Verdict::OnGrid
                        } else {
                            Verdict::OffGrid
                        }
                    }
                };
                Row::new(case.to_string(), hand.to_string(), history.to_string(), edge.to_string(), spr, legal, verdict)
            }
        }
    }

    /// Litmus edges are coarse labels (`O` = any raise, `!` = shove, `F` = fold, etc.).
    /// `parse_edge` returns a canonical Edge; we want any choice whose category matches.
    fn edge_matches(legal: &Edge, queried: &Edge) -> bool {
        if legal == queried {
            return true;
        }
        match (queried, legal) {
            (Edge::Raise(_), Edge::Raise(_)) => true,
            (Edge::Shove, Edge::Shove) => true,
            _ => false,
        }
    }
}
