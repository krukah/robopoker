//! Per-case evaluation: build typed inputs, call `Ops::policy`, compare
//! to the case's expected tolerance.
//!
//! Generic over an `Ops` impl so the same code serves the CLI binary
//! (direct DB) and HTTP handlers (server-side wrap).

use crate::catalog::{Catalog, build_witness, parse_edge};
use crate::ops::Ops;
use crate::schema::{Case, Direction, Expect, TestKind};
use cowboys::{ApiStrategy, Edge};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Fail,
    Skip,
    Error,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
            Self::Error => "ERROR",
        }
    }
}

/// Evaluation result for one case.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Outcome {
    pub case: Case,
    pub status: Status,
    pub detail: String,
    /// Per-state observations: (label, probability). Single-state kinds
    /// have one entry; pair_diff has 2; monotonic has N.
    pub observed: Vec<(String, Option<f32>)>,
}

pub async fn evaluate<O: Ops>(ops: &O, catalog: &Catalog<'_>, case: &Case) -> Outcome {
    match case.kind {
        TestKind::Single | TestKind::Exists => evaluate_single(ops, catalog, case).await,
        TestKind::PairDiff => evaluate_pair_diff(ops, catalog, case).await,
        TestKind::Monotonic => evaluate_monotonic(ops, catalog, case).await,
    }
}

fn merged_expect(catalog: &Catalog, case: &Case) -> Expect {
    Expect::merged(catalog.category_default(&case.category), case.expect.as_ref())
}

fn fmt_pct(p: Option<f32>) -> String {
    match p {
        Some(v) => format!("{:.1}%", v * 100.0),
        None => "—".to_string(),
    }
}

fn edge_prob(strategy: &ApiStrategy, edge: Edge) -> Option<f32> {
    let total: f32 = strategy.accumulated.values().copied().filter(|&v| v > 0.0).sum();
    if total <= 0.0 {
        return None;
    }
    let raw = strategy.accumulated.get(&edge).copied().unwrap_or(0.0).max(0.0);
    Some(raw / total)
}

async fn lookup_prob<O: Ops>(
    ops: &O,
    catalog: &Catalog<'_>,
    hand_ref: &str,
    history_ref: &str,
    edge_str: &str,
) -> anyhow::Result<Option<f32>> {
    let witness = build_witness(catalog, hand_ref, history_ref)?;
    let edge = parse_edge(edge_str)?;
    let strategy = ops.policy(witness).await?;
    Ok(strategy.as_ref().and_then(|s| edge_prob(s, edge)))
}

async fn evaluate_single<O: Ops>(ops: &O, catalog: &Catalog<'_>, case: &Case) -> Outcome {
    let outcome = |status: Status, detail: String, observed: Vec<(String, Option<f32>)>| Outcome {
        case: case.clone(),
        status,
        detail,
        observed,
    };

    let Some(hand_ref) = case.hand.as_deref() else {
        return outcome(Status::Error, "single/exists kind requires `hand` field".into(), vec![]);
    };

    let prob = match lookup_prob(ops, catalog, hand_ref, &case.history, &case.edge).await {
        Ok(p) => p,
        Err(e) => return outcome(Status::Error, format!("{e}"), vec![]),
    };

    let observed = vec![(hand_ref.to_string(), prob)];
    let Some(prob) = prob else {
        return outcome(Status::Skip, "no policy (state unvisited / empty blueprint)".into(), observed);
    };

    let expect = merged_expect(catalog, case);
    let max = expect.acceptable_max;
    let min = expect.acceptable_min;

    if let Some(m) = max
        && prob > m
    {
        return outcome(Status::Fail, format!("{} > {}", fmt_pct(Some(prob)), fmt_pct(Some(m))), observed);
    }
    if let Some(m) = min
        && prob < m
    {
        return outcome(Status::Fail, format!("{} < {}", fmt_pct(Some(prob)), fmt_pct(Some(m))), observed);
    }

    let bound = match (max, min) {
        (Some(mx), Some(mn)) => format!("in [{}, {}]", fmt_pct(Some(mn)), fmt_pct(Some(mx))),
        (Some(mx), None) => format!("≤{}", fmt_pct(Some(mx))),
        (None, Some(mn)) => format!("≥{}", fmt_pct(Some(mn))),
        (None, None) => "no bound".to_string(),
    };
    outcome(Status::Pass, format!("{} ({})", fmt_pct(Some(prob)), bound), observed)
}

async fn evaluate_pair_diff<O: Ops>(ops: &O, catalog: &Catalog<'_>, case: &Case) -> Outcome {
    let mk = |status, detail, observed| Outcome {
        case: case.clone(),
        status,
        detail,
        observed,
    };

    let Some(hands) = case.hands.as_ref() else {
        return mk(Status::Error, "pair_diff requires `hands` (length 2)".into(), vec![]);
    };
    if hands.len() != 2 {
        return mk(Status::Error, format!("pair_diff requires exactly 2 hands, got {}", hands.len()), vec![]);
    }

    let mut observed = Vec::with_capacity(2);
    for h in hands {
        let prob = match lookup_prob(ops, catalog, h, &case.history, &case.edge).await {
            Ok(p) => p,
            Err(e) => return mk(Status::Error, format!("{e}"), observed.clone()),
        };
        observed.push((h.clone(), prob));
    }

    let missing: Vec<String> = observed
        .iter()
        .filter(|(_, p)| p.is_none())
        .map(|(l, _)| l.clone())
        .collect();
    if !missing.is_empty() {
        return mk(Status::Skip, format!("missing data: {}", missing.join(", ")), observed);
    }

    let a = observed[0].1.unwrap();
    let b = observed[1].1.unwrap();
    let diff = (a - b).abs();
    let expect = merged_expect(catalog, case);
    let Some(bound) = expect.max_abs_diff else {
        return mk(Status::Error, "pair_diff missing max_abs_diff".into(), observed);
    };

    let obs_str = format!("{}={}  {}={}", observed[0].0, fmt_pct(observed[0].1), observed[1].0, fmt_pct(observed[1].1));
    if diff > bound {
        return mk(
            Status::Fail,
            format!("|Δ|={} > {} ({obs_str})", fmt_pct(Some(diff)), fmt_pct(Some(bound))),
            observed,
        );
    }
    mk(
        Status::Pass,
        format!("|Δ|={} ≤ {} ({obs_str})", fmt_pct(Some(diff)), fmt_pct(Some(bound))),
        observed,
    )
}

async fn evaluate_monotonic<O: Ops>(ops: &O, catalog: &Catalog<'_>, case: &Case) -> Outcome {
    let mk = |status, detail, observed| Outcome {
        case: case.clone(),
        status,
        detail,
        observed,
    };

    // Either N hands at one history, or one hand across N histories.
    // Tuple shape: (display_label, hand_ref, history_ref).
    let points: Vec<(String, String, String)> = match (&case.hands, &case.histories, &case.hand) {
        (Some(hs), None, _) if hs.len() >= 2 => hs
            .iter()
            .map(|h| (h.clone(), h.clone(), case.history.clone()))
            .collect(),
        (None, Some(hs), Some(hand)) if hs.len() >= 2 => hs
            .iter()
            .map(|h| (h.rsplit('.').next().unwrap_or(h).into(), hand.clone(), h.clone()))
            .collect(),
        _ => {
            return mk(
                Status::Error,
                "monotonic needs `hands[≥2]` with `history`, or `hand` with `histories[≥2]`".into(),
                vec![],
            );
        }
    };

    let mut observed = Vec::with_capacity(points.len());
    for (label, hand_ref, history_ref) in &points {
        let prob = match lookup_prob(ops, catalog, hand_ref, history_ref, &case.edge).await {
            Ok(p) => p,
            Err(e) => return mk(Status::Error, format!("{e}"), observed.clone()),
        };
        observed.push((label.clone(), prob));
    }

    let missing: Vec<String> = observed
        .iter()
        .filter(|(_, p)| p.is_none())
        .map(|(l, _)| l.clone())
        .collect();
    if !missing.is_empty() {
        return mk(Status::Skip, format!("missing data: {}", missing.join(", ")), observed);
    }

    let probs: Vec<f32> = observed.iter().map(|(_, p)| p.unwrap()).collect();
    let expect = merged_expect(catalog, case);
    let direction = expect.direction.unwrap_or(Direction::Increasing);
    let eps = 1e-6_f32;
    let monotonic = match direction {
        Direction::Increasing => probs.windows(2).all(|w| w[0] <= w[1] + eps),
        Direction::Decreasing => probs.windows(2).all(|w| w[0] >= w[1] - eps),
    };
    let dir_label = match direction {
        Direction::Increasing => "increasing",
        Direction::Decreasing => "decreasing",
    };

    let obs_str = observed
        .iter()
        .map(|(l, p)| format!("{l}={}", fmt_pct(*p)))
        .collect::<Vec<_>>()
        .join(" → ");
    if !monotonic {
        return mk(Status::Fail, format!("non-monotonic ({dir_label}): {obs_str}"), observed);
    }
    mk(Status::Pass, format!("monotonic {dir_label}: {obs_str}"), observed)
}
