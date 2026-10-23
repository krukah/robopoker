//! Family expansion: a `Family` template + matrix → N concrete `Case`s.
//!
//! Cartesian product over the matrix axes. Known matrix keys: `hand`      — `Vec<String>`,
//! fills `case.hand` `history`   — `Vec<String>`, fills `case.history` `edge`      —
//! `Vec<String>`, fills `case.edge` `hand_pair` — `Vec<[String; 2]>`, fills `case.hands`
//! (length-2)

use crate::{Case, Family, Scenarios, TestKind};
use std::collections::HashMap;

/// Namespace for the compose routines.
pub struct Compose;

impl Compose {
    /// Combine inline `tests` with expanded `families` into a flat list.
    pub fn resolve(scenarios: &Scenarios) -> anyhow::Result<Vec<Case>> {
        Ok(scenarios
            .tests()
            .iter()
            .cloned()
            .chain(
                scenarios
                    .families()
                    .iter()
                    .map(Compose::expand)
                    .collect::<anyhow::Result<Vec<_>>>()?
                    .into_iter()
                    .flatten(),
            )
            .collect())
    }

    fn expand(family: &Family) -> anyhow::Result<Vec<Case>> {
        let matrix = family.matrix();
        if matrix.is_empty() {
            // No matrix → produce a single case.
            return Ok(vec![Compose::instantiate(family, &HashMap::new())?]);
        }

        // Compute Cartesian product over matrix axes.
        let keys = matrix.keys().collect::<Vec<&String>>();
        let value_lists = keys
            .iter()
            .map(|k| &matrix[*k])
            .collect::<Vec<&Vec<serde_json::Value>>>();

        let mut cases = Vec::new();
        for combo in Compose::cartesian(&value_lists) {
            let assignments = keys
                .iter()
                .copied()
                .map(String::as_str)
                .zip(combo.iter().copied())
                .collect::<HashMap<&str, &serde_json::Value>>();
            cases.push(Compose::instantiate(family, &assignments)?);
        }
        Ok(cases)
    }

    fn cartesian<'lists>(lists: &'lists [&'lists Vec<serde_json::Value>]) -> Vec<Vec<&'lists serde_json::Value>> {
        if lists.is_empty() {
            return vec![vec![]];
        }
        let head = lists[0];
        let rest = Compose::cartesian(&lists[1..]);
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

    fn instantiate(family: &Family, assignments: &HashMap<&str, &serde_json::Value>) -> anyhow::Result<Case> {
        if family.hands_seq().is_some() && family.histories_seq().is_some() {
            anyhow::bail!("family `{}`: `hands_seq` and `histories_seq` are mutually exclusive", family.name_template())
        }
        let case = assignments.iter().try_fold(
            Case {
                name: String::new(),
                category: family.category().to_string(),
                kind: match family.hands_seq().or(family.histories_seq()) {
                    Some(_) if family.kind() == TestKind::Single => TestKind::Monotonic,
                    _ => family.kind(),
                },
                edge: family.edge().to_string(),
                hand: family.hand().cloned(),
                hands: family.hands_seq().or(family.hands()).cloned(),
                history: family.history().cloned().unwrap_or_default(),
                histories: family.histories_seq().cloned(),
                expect: family.expect().cloned(),
                desired: family.desired().cloned(),
                diagnosis_if_violated: family.diagnosis_if_violated().cloned(),
                historical: family.historical().cloned(),
            },
            |case, (key, value)| Compose::substitute(family, case, key, value),
        )?;
        Ok(Case {
            name: Compose::render_name(family.name_template(), assignments, &case),
            ..case
        })
    }

    fn substitute(family: &Family, case: Case, key: &str, value: &serde_json::Value) -> anyhow::Result<Case> {
        match key {
            "hand" => Ok(Case {
                hand: Some(Compose::as_str(value, "hand")?),
                ..case
            }),
            "hand_pair" => Ok(Case {
                hands: Some(Compose::as_pair(value, "hand_pair")?),
                kind: match case.kind() {
                    TestKind::Single => TestKind::PairDiff,
                    kind => kind,
                },
                ..case
            }),
            "history" => Ok(Case {
                history: Compose::as_str(value, "history")?,
                ..case
            }),
            "edge" => Ok(Case {
                edge: Compose::as_str(value, "edge")?,
                ..case
            }),
            other => anyhow::bail!("unknown matrix key `{other}` in family `{}`", family.name_template()),
        }
    }

    fn as_str(v: &serde_json::Value, key: &str) -> anyhow::Result<String> {
        v.as_str()
            .map(std::string::ToString::to_string)
            .ok_or_else(|| anyhow::anyhow!("matrix `{key}` value must be string, got {v}"))
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
                    .map(std::string::ToString::to_string)
                    .ok_or_else(|| anyhow::anyhow!("matrix `{key}` element must be string"))
            })
            .collect()
    }

    /// Substitute `{hand}`, `{hands_a}`, `{hands_b}`, `{history}`, `{edge}` etc
    /// in the template using the per-instance assignments and resolved case fields.
    fn render_name(template: &str, assignments: &HashMap<&str, &serde_json::Value>, case: &Case) -> String {
        let mut out = template.to_string();

        if let Some(h) = assignments.get("hand").copied().and_then(serde_json::Value::as_str) {
            out = out.replace("{hand}", h);
        } else if let Some(h) = case.hand() {
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
        } else if let Some(hands) = case.hands() {
            if hands.len() >= 2 {
                out = out.replace("{hands_a}", &hands[0]);
                out = out.replace("{hands_b}", &hands[1]);
            }
            out = out.replace("{hands}", &hands.join(","));
        }

        // {history} — last segment after dot
        let history_full = assignments
            .get("history")
            .copied()
            .and_then(serde_json::Value::as_str)
            .unwrap_or(case.history());
        let history_short = history_full.rsplit('.').next().unwrap_or(history_full);
        out = out.replace("{history}", history_short);

        let edge = assignments
            .get("edge")
            .copied()
            .and_then(serde_json::Value::as_str)
            .unwrap_or(case.edge());
        out = out.replace("{edge}", edge);

        out
    }
}
