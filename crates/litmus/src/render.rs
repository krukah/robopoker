//! Markdown report rendering. Mirrors the Python runner's output so
//! both produce comparable reports.

use crate::evaluate::{Outcome, Status};
use crate::schema::{CategoryDef, Scenarios};
use rbp_gameplay::{ApiGridUsage, ApiStatus};
use std::collections::BTreeMap;

pub fn render(
    api_label: &str,
    status: Option<&ApiStatus>,
    scenarios: &Scenarios,
    outcomes: &[Outcome],
    grid_usage: Option<&[ApiGridUsage]>,
) -> String {
    let mut out = String::new();
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S");
    out.push_str(&format!("# Blueprint litmus — {ts}Z\n\n"));
    out.push_str(&format!("**API**: `{api_label}`\n\n"));
    if let Some(s) = status {
        let exp = s
            .exploit
            .map(|v| format!("{v:.1}"))
            .unwrap_or_else(|| "n/a".to_string());
        out.push_str(&format!(
            "**Blueprint**: epoch={}  infosets={}  sum_regret={exp}\n\n",
            s.epoch, s.infosets,
        ));
    }

    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut skip = 0usize;
    let mut error = 0usize;
    for o in outcomes {
        match o.status {
            Status::Pass => pass += 1,
            Status::Fail => fail += 1,
            Status::Skip => skip += 1,
            Status::Error => error += 1,
        }
    }
    out.push_str("## Summary\n\n");
    out.push_str(&format!("- **PASS**: {pass}\n"));
    out.push_str(&format!("- **FAIL**: {fail}\n"));
    out.push_str(&format!("- **SKIP**: {skip}\n"));
    if error > 0 {
        out.push_str(&format!("- **ERROR**: {error}\n"));
    }
    out.push_str(&format!("- **Total**: {}\n\n", outcomes.len()));

    // Group by category
    let mut by_cat: BTreeMap<&str, Vec<&Outcome>> = BTreeMap::new();
    for o in outcomes {
        by_cat.entry(o.case.category.as_str()).or_default().push(o);
    }

    for (cat, items) in &by_cat {
        out.push_str(&format!("## Category: `{cat}`\n\n"));
        if let Some(meta) = scenarios.categories.get(*cat) {
            render_category_intro(meta, &mut out);
        }
        out.push_str("| Case | Kind | Edge | Status | Detail |\n");
        out.push_str("|---|---|---|---|---|\n");
        for o in items {
            let kind = format!("{:?}", o.case.kind).to_lowercase();
            out.push_str(&format!(
                "| {} | `{}` | `{}` | **{}** | {} |\n",
                o.case.name,
                kind,
                o.case.edge,
                o.status.label(),
                escape_pipes(&o.detail),
            ));
        }
        out.push('\n');

        for o in items {
            if !matches!(o.status, Status::Pass) {
                render_failure_block(o, &mut out);
            }
        }
    }

    if let Some(rows) = grid_usage {
        render_grid_usage(rows, &mut out);
    }

    out.push_str("## Methodology\n\n");
    out.push_str(
        "See [`scripts/litmus/README.md`](../scripts/litmus/README.md) and \
         [`bin/litmus/scenarios.json`](../bin/litmus/scenarios.json). The \
         compositional schema uses named hands/histories/categories; tests \
         reference them rather than inlining. Families auto-expand a template \
         across a matrix.\n\n",
    );
    out
}

fn render_category_intro(meta: &CategoryDef, out: &mut String) {
    out.push_str(&format!("_{}_\n\n", meta.description));
}

fn render_failure_block(o: &Outcome, out: &mut String) {
    out.push_str(&format!("### {}\n\n", o.case.name));
    if let Some(desired) = &o.case.desired {
        out.push_str(&format!("**Desired**: {desired}\n\n"));
    }
    if matches!(o.status, Status::Fail | Status::Error) {
        if let Some(diag) = &o.case.diagnosis_if_violated {
            out.push_str(&format!("**Diagnosis**: {diag}\n\n"));
        }
    }
    if let Some(historical) = &o.case.historical {
        if let Some(ctx) = &historical.context {
            out.push_str(&format!("**Historical**: {ctx}\n\n"));
        }
    }
}

fn render_grid_usage(rows: &[ApiGridUsage], out: &mut String) {
    out.push_str("## Aggregate `!` (shove) frequency by street\n\n");
    out.push_str(
        "Two views: `avg_freq` overweights low-visit decisions; \
         `weighted_freq` approximates real-game shove rate.\n\n",
    );
    out.push_str("| street | n_decisions | n_dominant | % dominant | avg_freq | weighted_freq |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for street in ["preflop", "flop", "turn", "river"] {
        for r in rows {
            if r.street == street && r.edge == "!" {
                let n_dec = r.n_decisions_with_edge;
                let n_dom = r.n_dominant;
                let dom_pct = if n_dec > 0 {
                    (n_dom as f64) / (n_dec as f64) * 100.0
                } else {
                    0.0
                };
                out.push_str(&format!(
                    "| {} | {} | {} | {:.1}% | {:.1}% | {:.2}% |\n",
                    street,
                    n_dec,
                    n_dom,
                    dom_pct,
                    r.avg_freq * 100.0,
                    r.weighted_freq * 100.0,
                ));
                break;
            }
        }
    }
    out.push('\n');
}

fn escape_pipes(s: &str) -> String {
    s.replace('|', "\\|")
}
