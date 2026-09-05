//! Tier 1 report: observed bet sizes against the live `Size` grid.
//!
//! Read the table cell by cell. `medErr` is how far the median raise sits from
//! its nearest anchor — the size information translation discards before the
//! policy is consulted. `>max` is worse than a large `medErr`: those are bets
//! the grid can only represent by shrinking, so the abstraction cannot express
//! them at all. `dead` names anchors nothing snapped to — branches of the game
//! tree the corpus never plays into.

use crate::render::Table;
use crate::render::commas;
use phh::Calibration;
use phh::Record;

pub fn run(records: &[Record], player: Option<&str>) -> anyhow::Result<()> {
    let bblind = records
        .iter()
        .flat_map(|r| r.blinds().iter().copied())
        .max()
        .ok_or_else(|| anyhow::anyhow!("no hands"))?;
    let outcomes = records
        .iter()
        .map(|record| Ok((record, phh::replay(record)?)))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let nodes = outcomes
        .iter()
        .flat_map(|(record, outcome)| {
            let seat = player.and_then(|name| record.seat_of(name));
            outcome
                .nodes()
                .iter()
                .filter(move |node| player.is_none() || seat == Some(node.seat()))
        })
        .collect::<Vec<_>>();
    let calibration = phh::calibrate(nodes, bblind);
    println!(
        "## Bet-size calibration — {} ({} raises, big blind {bblind})\n",
        player.unwrap_or("all players"),
        commas(calibration.n())
    );
    print!("{}", table(&calibration));
    println!(
        "\nSizes use the engine's convention: chips put in now over the pot before the action,\n\
         except the preflop opening row, which counts big blinds put in now.\n\
         `on grid` is the share landing within 15% of some anchor. Shoves bypass the grid\n\
         via Edge::Shove and are excluded from every fit."
    );
    Ok(())
}

fn table(calibration: &Calibration) -> Table {
    let mut table = Table::new(
        &[
            "cell",
            "n",
            "shove",
            "p25",
            "p50",
            "p75",
            "medErr",
            ">max",
            "<min",
            "on grid",
            "anchor usage",
        ],
        &[9, 6, 6, 7, 7, 7, 7, 6, 6, 8, 0],
    );
    for cell in calibration.cells().iter().filter(|c| c.n() > 0) {
        let unit = if cell.opening() { "bb" } else { "x" };
        let show = |q: f64| cell.quantile(q).map_or("—".into(), |v| format!("{v:.2}{unit}"));
        table.push(vec![
            format!("{:?}/{}", cell.street(), if cell.depth() < 2 { cell.depth().to_string() } else { "N".into() }),
            commas(cell.n()),
            format!("{:.0}%", 100.0 * cell.shoves() as f64 / cell.n() as f64),
            show(0.25),
            show(0.50),
            show(0.75),
            cell.median_error().map_or("—".into(), |e| format!("{:.1}%", 100.0 * e)),
            format!("{:.0}%", 100.0 * cell.over()),
            format!("{:.0}%", 100.0 * cell.under()),
            format!("{:.0}%", 100.0 * cell.on_grid()),
            cell.usage()
                .iter()
                .map(|(label, share)| format!("{label} {:.0}%", 100.0 * share))
                .collect::<Vec<_>>()
                .join("  "),
        ]);
    }
    table
}
