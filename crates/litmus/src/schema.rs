//! Serde types for `scenarios.json`.
//!
//! Mirrors the v4 compositional schema: top-level `hands` / `histories` /
//! `categories` libraries, plus `tests` (concrete cases) and `families`
//! (matrix templates that expand to multiple cases).
//!
//! See `scripts/litmus/README.md` for the human-facing schema documentation.

use super::*;

/// Namespace for the schema routines.
pub struct Schema;

impl Schema {
    /// Load and parse `scenarios.json` from a path.
    pub fn load(path: &std::path::Path) -> anyhow::Result<Scenarios> {
        let text = std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("read {}: {}", path.display(), e))?;
        let scenarios: Scenarios =
            serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("parse {}: {}", path.display(), e))?;
        Ok(scenarios)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Schema {
        fn path() -> std::path::PathBuf {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bin/litmus/scenarios.json")
        }
    }

    /// Parse the checked-in catalog. Catches schema drift and JSON typos
    /// without needing a database connection.
    #[test]
    fn scenarios_json_parses() {
        let scenarios = Schema::load(&Schema::path()).expect("scenarios.json must parse");
        assert!(!scenarios.hands().is_empty());
        assert!(!scenarios.histories().is_empty());
        assert!(!scenarios.categories().is_empty());
    }

    /// Build a witness against every history that declares `expected_spr` and confirm the
    /// runner's bucket validator agrees.
    #[test]
    fn expected_spr_declarations_match_geometry() {
        let scenarios = Schema::load(&Schema::path()).unwrap();
        let catalog = Catalog::new(&scenarios);
        let n = scenarios
            .histories()
            .iter()
            .flat_map(|(street, hs)| hs.iter().map(move |(name, def)| (street, name, def)))
            .filter(|(_, _, def)| def.expected_spr().is_some())
            .map(|(street, name, _)| {
                let dotted = format!("{street}.{name}");
                Catalog::build_witness(&catalog, "AKo", &dotted)
                    .unwrap_or_else(|e| panic!("SPR history `{dotted}`: {e}"));
            })
            .count();
        assert!(n > 0, "expected at least one history with expected_spr");
    }
}
