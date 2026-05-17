//! Family expansion: a `Family` template + matrix → N concrete `Case`s.
//!
//! Cartesian product over the matrix axes. Known matrix keys:
//!   `hand`      — `Vec<String>`, fills `case.hand`
//!   `history`   — `Vec<String>`, fills `case.history`
//!   `edge`      — `Vec<String>`, fills `case.edge`
//!   `hand_pair` — `Vec<[String; 2]>`, fills `case.hands` (length-2)
//!
//! Family-level `hands_seq` is promoted to `case.hands` on every instance
//! (no per-instance variation; sets up monotonic kind).

use crate::schema::{Case, Family, Scenarios, TestKind};
use std::collections::HashMap;

/// Combine inline `tests` with expanded `families` into a flat list.
pub fn resolve(scenarios: &Scenarios) -> anyhow::Result<Vec<Case>> {
    let mut out: Vec<Case> = scenarios.tests.clone();
    for family in &scenarios.families {
        out.extend(expand(family)?);
    }
    Ok(out)
}

fn expand(family: &Family) -> anyhow::Result<Vec<Case>> {
    let matrix = &family.matrix;
    if matrix.is_empty() {
        // No matrix → produce a single case.
        return Ok(vec![instantiate(family, &HashMap::new())?]);
    }

    // Compute Cartesian product over matrix axes.
    let keys: Vec<&String> = matrix.keys().collect();
    let value_lists: Vec<&Vec<serde_json::Value>> = keys.iter().map(|k| &matrix[*k]).collect();

    let mut cases = Vec::new();
    for combo in cartesian(&value_lists) {
        let assignments: HashMap<&str, &serde_json::Value> = keys
            .iter()
            .map(|k| k.as_str())
            .zip(combo.iter().copied())
            .collect();
        cases.push(instantiate(family, &assignments)?);
    }
    Ok(cases)
}

fn cartesian<'a>(lists: &'a [&'a Vec<serde_json::Value>]) -> Vec<Vec<&'a serde_json::Value>> {
    if lists.is_empty() {
        return vec![vec![]];
    }
    let head = lists[0];
    let rest = cartesian(&lists[1..]);
    let mut out = Vec::with_capacity(head.len() * rest.len());
    for h in head {
        for tail in &rest {
            let mut combined = vec![h];
            combined.extend(tail);
            out.push(combined);
        }
    }
    out
}

fn instantiate(
    family: &Family,
    assignments: &HashMap<&str, &serde_json::Value>,
) -> anyhow::Result<Case> {
    let mut case = Case {
        name: String::new(),
        category: family.category.clone(),
        kind: family.kind,
        edge: family.edge.clone(),
        hand: family.hand.clone(),
        hands: family.hands.clone(),
        history: family.history.clone().unwrap_or_default(),
        expect: family.expect.clone(),
        desired: family.desired.clone(),
        diagnosis_if_violated: family.diagnosis_if_violated.clone(),
        historical: family.historical.clone(),
    };

    // Promote hands_seq → hands (monotonic kind sets up here).
    if let Some(seq) = &family.hands_seq {
        case.hands = Some(seq.clone());
        if case.kind == TestKind::Single {
            case.kind = TestKind::Monotonic;
        }
    }

    // Apply matrix substitutions.
    for (key, value) in assignments {
        match *key {
            "hand" => {
                let s = as_str(value, "hand")?;
                case.hand = Some(s.to_string());
            }
            "hand_pair" => {
                let pair = as_pair(value, "hand_pair")?;
                case.hands = Some(pair);
                if case.kind == TestKind::Single {
                    case.kind = TestKind::PairDiff;
                }
            }
            "history" => {
                let s = as_str(value, "history")?;
                case.history = s.to_string();
            }
            "edge" => {
                let s = as_str(value, "edge")?;
                case.edge = s.to_string();
            }
            other => {
                anyhow::bail!(
                    "unknown matrix key `{other}` in family `{}`",
                    family.name_template
                );
            }
        }
    }

    case.name = render_name(&family.name_template, assignments, &case);

    Ok(case)
}

fn as_str(v: &serde_json::Value, key: &str) -> anyhow::Result<String> {
    v.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("matrix `{key}` value must be string, got {}", v))
}

fn as_pair(v: &serde_json::Value, key: &str) -> anyhow::Result<Vec<String>> {
    let arr = v
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("matrix `{key}` value must be array of 2 strings"))?;
    if arr.len() != 2 {
        anyhow::bail!("matrix `{key}` value must be length 2, got {}", arr.len());
    }
    arr.iter()
        .map(|e| {
            e.as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("matrix `{key}` element must be string"))
        })
        .collect()
}

/// Substitute `{hand}`, `{hands_a}`, `{hands_b}`, `{history}`, `{edge}` etc
/// in the template using the per-instance assignments and resolved case fields.
fn render_name(
    template: &str,
    assignments: &HashMap<&str, &serde_json::Value>,
    case: &Case,
) -> String {
    let mut out = template.to_string();

    // {hand}
    if let Some(h) = assignments.get("hand").and_then(|v| v.as_str()) {
        out = out.replace("{hand}", h);
    } else if let Some(h) = &case.hand {
        out = out.replace("{hand}", h);
    }

    // {hands_a}, {hands_b}, {hands}
    if let Some(pair_v) = assignments.get("hand_pair") {
        if let Some(arr) = pair_v.as_array()
            && arr.len() == 2
            && let (Some(a), Some(b)) = (arr[0].as_str(), arr[1].as_str())
        {
            out = out.replace("{hands_a}", a);
            out = out.replace("{hands_b}", b);
            out = out.replace("{hands}", &format!("{a},{b}"));
        }
    } else if let Some(hands) = &case.hands {
        if hands.len() >= 2 {
            out = out.replace("{hands_a}", &hands[0]);
            out = out.replace("{hands_b}", &hands[1]);
        }
        out = out.replace("{hands}", &hands.join(","));
    }

    // {history} — last segment after dot
    let history_full = assignments
        .get("history")
        .and_then(|v| v.as_str())
        .unwrap_or(&case.history);
    let history_short = history_full.rsplit('.').next().unwrap_or(history_full);
    out = out.replace("{history}", history_short);

    // {edge}
    let edge = assignments
        .get("edge")
        .and_then(|v| v.as_str())
        .unwrap_or(&case.edge);
    out = out.replace("{edge}", edge);

    out
}
