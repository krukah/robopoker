//! Per-case evaluation: build typed inputs, call `Ops::policy`, compare
//! to the case's expected tolerance.
//!
//! Generic over an `Ops` impl so the same code serves the CLI binary
//! (direct DB) and HTTP handlers (server-side wrap).
use super::*;

use crate::*;
use crate::{Case, Direction, Expect, TestKind};
use kicker::{ApiStrategy, Edge};

/// Namespace for the evaluate routines.
pub struct Evaluate;

impl Evaluate {
    pub async fn evaluate<O>(ops: &O, catalog: &Catalog<'_>, case: &Case) -> Outcome
    where
        O: Ops,
    {
        match case.kind() {
            TestKind::Single | TestKind::Exists => Evaluate::evaluate_single(ops, catalog, case).await,
            TestKind::PairDiff => Evaluate::evaluate_pair_diff(ops, catalog, case).await,
            TestKind::Monotonic => Evaluate::evaluate_monotonic(ops, catalog, case).await,
        }
    }

    fn merged_expect(catalog: &Catalog, case: &Case) -> Expect {
        Expect::merged(catalog.category_default(case.category()), case.expect())
    }

    fn fmt_pct(p: Option<f32>) -> String {
        match p {
            Some(v) => format!("{:.1}%", v * 100.0),
            None => "—".to_string(),
        }
    }

    fn edge_prob(strategy: &ApiStrategy, edge: Edge) -> Option<f32> {
        let total = strategy
            .accumulated()
            .values()
            .copied()
            .filter(|&v| v > 0.0)
            .sum::<f32>();
        if total <= 0.0 {
            return None;
        }
        let raw = strategy.accumulated().get(&edge).copied().unwrap_or(0.0).max(0.0);
        Some(raw / total)
    }

    async fn lookup_prob<O>(
        ops: &O,
        catalog: &Catalog<'_>,
        hand_ref: &str,
        history_ref: &str,
        edge_str: &str,
    ) -> anyhow::Result<Option<f32>>
    where
        O: Ops,
    {
        let witness = Catalog::build_witness(catalog, hand_ref, history_ref)?;
        let edge = Catalog::parse_edge(edge_str)?;
        let strategy = ops.policy(witness).await?;
        Ok(strategy.as_ref().and_then(|s| Evaluate::edge_prob(s, edge)))
    }

    async fn evaluate_single<O>(ops: &O, catalog: &Catalog<'_>, case: &Case) -> Outcome
    where
        O: Ops,
    {
        let outcome = |status: Status, detail: String, observed: Vec<(String, Option<f32>)>| Outcome {
            case: case.clone(),
            status,
            detail,
            observed,
        };

        let Some(hand_ref) = case.hand() else {
            return outcome(Status::Error, "single/exists kind requires `hand` field".into(), vec![]);
        };

        let prob = match Evaluate::lookup_prob(ops, catalog, hand_ref, case.history(), case.edge()).await {
            Ok(p) => p,
            Err(e) => return outcome(Status::Error, format!("{e}"), vec![]),
        };

        let observed = vec![(hand_ref.to_string(), prob)];
        let Some(prob) = prob else {
            return outcome(Status::Skip, "no policy (state unvisited / empty blueprint)".into(), observed);
        };

        let expect = Evaluate::merged_expect(catalog, case);
        let max = expect.acceptable_max();
        let min = expect.acceptable_min();

        if let Some(m) = max
            && prob > m
        {
            return outcome(
                Status::Fail,
                format!("{} > {}", Evaluate::fmt_pct(Some(prob)), Evaluate::fmt_pct(Some(m))),
                observed,
            );
        }
        if let Some(m) = min
            && prob < m
        {
            return outcome(
                Status::Fail,
                format!("{} < {}", Evaluate::fmt_pct(Some(prob)), Evaluate::fmt_pct(Some(m))),
                observed,
            );
        }

        let bound = match (max, min) {
            (Some(mx), Some(mn)) => format!("in [{}, {}]", Evaluate::fmt_pct(Some(mn)), Evaluate::fmt_pct(Some(mx))),
            (Some(mx), None) => format!("≤{}", Evaluate::fmt_pct(Some(mx))),
            (None, Some(mn)) => format!("≥{}", Evaluate::fmt_pct(Some(mn))),
            (None, None) => "no bound".to_string(),
        };
        outcome(Status::Pass, format!("{} ({})", Evaluate::fmt_pct(Some(prob)), bound), observed)
    }

    async fn evaluate_pair_diff<O>(ops: &O, catalog: &Catalog<'_>, case: &Case) -> Outcome
    where
        O: Ops,
    {
        let mk = |status, detail, observed| Outcome {
            case: case.clone(),
            status,
            detail,
            observed,
        };

        let Some(hands) = case.hands() else {
            return mk(Status::Error, "pair_diff requires `hands` (length 2)".into(), vec![]);
        };
        if hands.len() != 2 {
            return mk(Status::Error, format!("pair_diff requires exactly 2 hands, got {}", hands.len()), vec![]);
        }

        let mut observed = Vec::with_capacity(2);
        for h in hands {
            let prob = match Evaluate::lookup_prob(ops, catalog, h, case.history(), case.edge()).await {
                Ok(p) => p,
                Err(e) => return mk(Status::Error, format!("{e}"), observed.clone()),
            };
            observed.push((h.clone(), prob));
        }

        let missing = observed
            .iter()
            .filter(|(_, p)| p.is_none())
            .map(|(l, _)| l.clone())
            .collect::<Vec<String>>();
        if !missing.is_empty() {
            return mk(Status::Skip, format!("missing data: {}", missing.join(", ")), observed);
        }

        let a = observed[0].1.unwrap();
        let b = observed[1].1.unwrap();
        let diff = (a - b).abs();
        let expect = Evaluate::merged_expect(catalog, case);
        let Some(bound) = expect.max_abs_diff() else {
            return mk(Status::Error, "pair_diff missing max_abs_diff".into(), observed);
        };

        let obs_str = format!(
            "{}={}  {}={}",
            observed[0].0,
            Evaluate::fmt_pct(observed[0].1),
            observed[1].0,
            Evaluate::fmt_pct(observed[1].1)
        );
        if diff > bound {
            return mk(
                Status::Fail,
                format!("|Δ|={} > {} ({obs_str})", Evaluate::fmt_pct(Some(diff)), Evaluate::fmt_pct(Some(bound))),
                observed,
            );
        }
        mk(
            Status::Pass,
            format!("|Δ|={} ≤ {} ({obs_str})", Evaluate::fmt_pct(Some(diff)), Evaluate::fmt_pct(Some(bound))),
            observed,
        )
    }

    async fn evaluate_monotonic<O>(ops: &O, catalog: &Catalog<'_>, case: &Case) -> Outcome
    where
        O: Ops,
    {
        let mk = |status, detail, observed| Outcome {
            case: case.clone(),
            status,
            detail,
            observed,
        };

        // Either N hands at one history, or one hand across N histories.
        // Tuple shape: (display_label, hand_ref, history_ref).
        let points: Vec<(String, String, String)> = match (case.hands(), case.histories(), case.hand()) {
            (Some(hs), None, _) if hs.len() >= 2 => hs
                .iter()
                .map(|h| (h.clone(), h.clone(), case.history().to_string()))
                .collect(),
            (None, Some(hs), Some(hand)) if hs.len() >= 2 => hs
                .iter()
                .map(|h| (h.rsplit('.').next().unwrap_or(h).into(), hand.to_string(), h.clone()))
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
            let prob = match Evaluate::lookup_prob(ops, catalog, hand_ref, history_ref, case.edge()).await {
                Ok(p) => p,
                Err(e) => return mk(Status::Error, format!("{e}"), observed.clone()),
            };
            observed.push((label.clone(), prob));
        }

        let missing = observed
            .iter()
            .filter(|(_, p)| p.is_none())
            .map(|(l, _)| l.clone())
            .collect::<Vec<String>>();
        if !missing.is_empty() {
            return mk(Status::Skip, format!("missing data: {}", missing.join(", ")), observed);
        }

        let probs = observed.iter().map(|(_, p)| p.unwrap()).collect::<Vec<f32>>();
        let expect = Evaluate::merged_expect(catalog, case);
        let direction = expect.direction().unwrap_or(Direction::Increasing);
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
            .map(|(l, p)| format!("{l}={}", Evaluate::fmt_pct(*p)))
            .collect::<Vec<_>>()
            .join(" → ");
        if !monotonic {
            return mk(Status::Fail, format!("non-monotonic ({dir_label}): {obs_str}"), observed);
        }
        mk(Status::Pass, format!("monotonic {dir_label}: {obs_str}"), observed)
    }
}
