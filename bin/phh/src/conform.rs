//! Tier 0: does the corpus replay, and does the engine agree about its shape?
//!
//! Pure arithmetic: blinds, side pots, split pots, uncalled bets.
//!
//! computed without the engine. Catches a bad log or a bad reader.
//!
//! at engine chip scale. Chip amounts round, so only whose-turn-it-is, action legality, and
//! street boundaries are asserted.

use crate::render::Table;
use phh::Record;

/// Namespace for the conform routines.
pub(crate) struct Conform;

impl Conform {
    pub fn run(records: &[Record]) -> anyhow::Result<()> {
        let mut settled = 0usize;
        let mut ordered = 0usize;
        let mut walked = 0usize;
        let mut coerced = 0usize;
        let mut decisions = 0usize;
        let mut split = 0usize;
        let mut census = std::collections::BTreeMap::<&str, usize>::new();
        let mut dead = std::collections::BTreeMap::<i32, usize>::new();
        let mut failures = Vec::<String>::new();
        let mut structural = Vec::<String>::new();
        for record in records {
            let outcome = match phh::Replay::replay(record) {
                Ok(outcome) => outcome,
                Err(e) => {
                    failures.push(format!("{}: {e}", record.source()));
                    continue;
                }
            };
            settled += usize::from(outcome.agrees_with(record));
            split += usize::from(outcome.odd_chips());
            if !outcome.agrees_with(record) {
                failures.push(format!(
                    "{}: replay {:?} vs logged {:?}",
                    record.source(),
                    outcome.finishing(),
                    record.finishing_stacks()
                ));
            }
            ordered += usize::from(phh::Replay::turn_order(record).is_none());
            let shape = phh::Project::shape(record, &outcome);
            *census.entry(shape.label()).or_default() += 1;
            if let Some(reduction) = phh::Project::reduce(record, &outcome) {
                *dead.entry(reduction.dead()).or_default() += 1;
            }
            match phh::Trace::walk::<6>(record, &outcome) {
                Ok(trace) => {
                    decisions += trace.decisions();
                    coerced += trace.coerced();
                    walked += usize::from(trace.clean());
                    if !trace.clean() && structural.len() < 5 {
                        structural.push(format!(
                            "{}: misordered {:?} illegal {:?} stalled {:?} terminated {}",
                            record.source(),
                            trace.misordered(),
                            trace.illegal(),
                            trace.stalled(),
                            trace.terminated()
                        ));
                    }
                }
                Err(e) => structural.push(format!("{}: {e}", record.source())),
            }
        }
        let n = records.len();
        println!("## Conformance\n");
        let mut table = Table::new(&["check", "pass", "of", "rate"], &[24, 8, 8, 8]);
        table.push(vec![
            "settlement".into(),
            Table::commas(settled),
            Table::commas(n),
            Table::pct(settled, n),
        ]);
        table.push(vec![
            "turn order".into(),
            Table::commas(ordered),
            Table::commas(n),
            Table::pct(ordered, n),
        ]);
        table.push(vec![
            "engine structure".into(),
            Table::commas(walked),
            Table::commas(n),
            Table::pct(walked, n),
        ]);
        print!("{table}");
        println!(
            "\n{} decision nodes walked through GameN<6>; {} raises rounded to engine chip scale ({}).",
            Table::commas(decisions),
            Table::commas(coerced),
            Table::pct(coerced, decisions)
        );
        println!("{} hands settled a pot that split onto a half chip.", Table::commas(split));
        for line in failures.iter().take(5) {
            println!("  settlement: {line}");
        }
        if failures.len() > 5 {
            println!("  ... and {} more settlement failures", failures.len() - 5);
        }
        for line in &structural {
            println!("  structure: {line}");
        }
        println!("\n## Census — how much of this corpus our heads-up tree can reach\n");
        let mut table = Table::new(&["shape", "hands", "share"], &[16, 8, 8]);
        for (label, count) in &census {
            table.push(vec![(*label).to_string(), Table::commas(*count), Table::pct(*count, n)]);
        }
        print!("{table}");
        let reducible = census.get("heads-up").copied().unwrap_or(0);
        println!("\n## Dead money in the {} reducible pots\n", Table::commas(reducible));
        let mut table = Table::new(&["dead chips", "hands", "share"], &[12, 8, 8]);
        for (chips, count) in dead.iter().filter(|(_, c)| **c >= reducible / 100) {
            table.push(vec![chips.to_string(), Table::commas(*count), Table::pct(*count, reducible)]);
        }
        print!("{table}");
        println!("\n(rows under 1% of reducible pots elided; these are folded blinds and folded raises)");
        match (settled, ordered) {
            (s, o) if s == n && o == n => Ok(()),
            _ => anyhow::bail!("{} settlement and {} turn-order failures", n - settled, n - ordered),
        }
    }
}
