//! Serde types for `scenarios.json`.
//!
//! Mirrors the v4 compositional schema: top-level `hands` / `histories` /
//! `categories` libraries, plus `tests` (concrete cases) and `families`
//! (matrix templates that expand to multiple cases).
//!
//! See `scripts/litmus/README.md` for the human-facing schema documentation.

use fulcrum::Chips;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level scenarios document.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Scenarios {
    /// Schema docs and field descriptions; runner ignores.
    #[serde(default, rename = "_meta")]
    #[allow(dead_code)]
    pub meta: serde_json::Value,

    pub hands: HashMap<String, HandDef>,

    /// Two-level: street ("preflop"/"flop"/"turn"/"river") → name → entry.
    pub histories: HashMap<String, HashMap<String, HistoryDef>>,

    pub categories: HashMap<String, CategoryDef>,

    #[serde(default)]
    pub tests: Vec<Case>,

    #[serde(default)]
    pub families: Vec<Family>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HandDef {
    /// Hole-card encoding parsed by `Observation::try_from`. e.g. "AcKd".
    pub cards: String,
    #[allow(dead_code)]
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HistoryDef {
    /// "P0" or "P1" — parsed by `Turn`'s Deserialize impl.
    pub turn: String,
    /// Postflop-only: board template with `*` placeholder for hole cards.
    /// e.g. "* ~ Kh 7d 2c". Preflop entries omit this.
    #[serde(default, rename = "_seen")]
    pub seen_template: Option<String>,
    /// Voluntary action history. Each string parsed by `Action::try_from`.
    pub past: Vec<String>,
    /// Override default 100bb HU stacks. Use to probe the same action
    /// sequence at a different SPR bucket.
    #[serde(default)]
    pub stacks: Option<[Chips; 2]>,
    /// SPR-bucket assertion: runner ERRORs if the constructed witness lands
    /// outside this bucket. Catches drift when editing `stacks` or `past`.
    /// Values: "committed" | "low" | "mid" | "deep".
    #[serde(default)]
    pub expected_spr: Option<String>,
    #[allow(dead_code)]
    pub desc: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CategoryDef {
    #[allow(dead_code)]
    pub description: String,
    /// Per-category default tolerance; overridden by case-level `expect`.
    pub default_expect: Option<Expect>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TestKind {
    #[default]
    Single,
    PairDiff,
    Monotonic,
    Exists,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Increasing,
    Decreasing,
}

/// Tolerance specification. Different fields apply to different `kind`s.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Expect {
    pub acceptable_max: Option<f32>,
    pub acceptable_min: Option<f32>,
    pub max_abs_diff: Option<f32>,
    pub direction: Option<Direction>,
}

impl Expect {
    /// Merge category default with case override; case fields win.
    pub fn merged(default: Option<&Expect>, override_: Option<&Expect>) -> Expect {
        let mut out = default.cloned().unwrap_or_default();
        if let Some(o) = override_ {
            if o.acceptable_max.is_some() {
                out.acceptable_max = o.acceptable_max;
            }
            if o.acceptable_min.is_some() {
                out.acceptable_min = o.acceptable_min;
            }
            if o.max_abs_diff.is_some() {
                out.max_abs_diff = o.max_abs_diff;
            }
            if o.direction.is_some() {
                out.direction = o.direction;
            }
        }
        out
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Historical {
    #[allow(dead_code)]
    pub baseline_value: Option<f32>,
    pub context: Option<String>,
}

/// A concrete test case. Either inline in `tests` or expanded from a `Family`.
///
/// Field availability depends on `kind`:
/// - `single` / `exists`: requires `hand`, `history`
/// - `pair_diff`: requires `hands` (length 2), `history`
/// - `monotonic`: requires EITHER `hands` (length ≥2) + `history` (sweep hands
///   at one history) OR `hand` + `histories` (length ≥2) (sweep histories at
///   one hand — useful for SPR-bucket monotonicity).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Case {
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub kind: TestKind,
    pub edge: String,

    /// Single-state kinds.
    pub hand: Option<String>,
    /// Multi-state kinds (pair_diff has 2; monotonic has N).
    pub hands: Option<Vec<String>>,

    /// History reference, dotted: "preflop.bb_defends_2bb".
    #[serde(default)]
    pub history: String,
    /// Monotonic-over-histories: sweep N histories at a single hand.
    /// Mutually exclusive with `hands`.
    #[serde(default)]
    pub histories: Option<Vec<String>>,

    pub expect: Option<Expect>,
    #[allow(dead_code)]
    pub desired: Option<String>,
    pub diagnosis_if_violated: Option<String>,
    pub historical: Option<Historical>,
}

/// A family is a template + a matrix. Cartesian-expand to N concrete cases.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Family {
    pub name_template: String,
    pub category: String,
    #[serde(default)]
    pub kind: TestKind,
    pub edge: String,

    /// Concrete fields. Any field listed in `matrix` is filled per-instance.
    pub hand: Option<String>,
    pub hands: Option<Vec<String>>,
    /// Used by monotonic kind: a sequence of hand refs that gets promoted
    /// into `hands` on each instance (no per-instance variation needed).
    pub hands_seq: Option<Vec<String>>,
    /// Used by monotonic kind: a sequence of history refs swept at the
    /// family's single `hand`. Mutually exclusive with `hands_seq`.
    pub histories_seq: Option<Vec<String>>,

    pub history: Option<String>,
    pub expect: Option<Expect>,
    pub desired: Option<String>,
    pub diagnosis_if_violated: Option<String>,
    pub historical: Option<Historical>,

    /// Cartesian product expanded over these axes. Known keys:
    /// - `hand`: `Vec<String>`
    /// - `history`: `Vec<String>`
    /// - `edge`: `Vec<String>`
    /// - `hand_pair`: Vec<[String; 2]>
    #[serde(default)]
    pub matrix: HashMap<String, Vec<serde_json::Value>>,
}

/// Load and parse `scenarios.json` from a path.
pub fn load(path: &std::path::Path) -> anyhow::Result<Scenarios> {
    let text = std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read {}: {}", path.display(), e))?;
    let scenarios: Scenarios =
        serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("parse {}: {}", path.display(), e))?;
    Ok(scenarios)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{Catalog, build_witness};

    fn catalog_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bin/litmus/scenarios.json")
    }

    /// Parse the checked-in catalog. Catches schema drift and JSON typos
    /// without needing a database connection.
    #[test]
    fn scenarios_json_parses() {
        let scenarios = load(&catalog_path()).expect("scenarios.json must parse");
        assert!(!scenarios.hands.is_empty());
        assert!(!scenarios.histories.is_empty());
        assert!(!scenarios.categories.is_empty());
    }

    /// Build a witness against every history that declares `expected_spr` and
    /// confirm the runner's bucket validator agrees. Caught the chip-delta vs
    /// chip-target confusion in the original 3/4/5-bet sequences.
    #[test]
    fn expected_spr_declarations_match_geometry() {
        let scenarios = load(&catalog_path()).unwrap();
        let catalog = Catalog::new(&scenarios);
        let n = scenarios
            .histories
            .iter()
            .flat_map(|(street, hs)| hs.iter().map(move |(name, def)| (street, name, def)))
            .filter(|(_, _, def)| def.expected_spr.is_some())
            .map(|(street, name, _)| {
                let dotted = format!("{street}.{name}");
                build_witness(&catalog, "AKo", &dotted).unwrap_or_else(|e| panic!("SPR history `{dotted}`: {e}"));
            })
            .count();
        assert!(n > 0, "expected at least one history with expected_spr");
    }
}
