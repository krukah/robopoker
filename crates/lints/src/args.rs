use std::collections::HashSet;

use clap::Parser;
use lints::*;

#[derive(Parser)]
#[command(name = "lints", about = "Diff-scoped house-style linter for Rust and TypeScript")]
pub struct Args {
    /// Base ref to diff against; the merge base with HEAD is used.
    /// Defaults to origin/main, then main.
    #[arg(long)]
    base: Option<String>,

    /// Lint whole files instead of only diff-touched comment blocks
    #[arg(long)]
    all: bool,

    /// Also enforce a rule that is off by default
    #[arg(long = "enable", value_name = "RULE")]
    enable: Vec<String>,

    /// Report only these rules
    #[arg(long = "only", value_name = "RULE")]
    only: Vec<String>,

    /// Report every function's branch complexity instead of linting
    #[arg(long)]
    census: bool,

    /// Restrict linting to these path prefixes
    paths: Vec<String>,
}

impl Args {
    pub fn base(&self) -> Option<&str> {
        self.base.as_deref()
    }

    pub fn all(&self) -> bool {
        self.all
    }

    pub fn enable(&self) -> &[String] {
        &self.enable
    }

    pub fn only(&self) -> &[String] {
        &self.only
    }

    pub fn census(&self) -> bool {
        self.census
    }

    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    pub fn selected(path: &str, prefixes: &[String]) -> bool {
        prefixes.is_empty()
            || prefixes.iter().any(|prefix| {
                let prefix = prefix.trim_end_matches('/');
                path == prefix || path.starts_with(&format!("{prefix}/"))
            })
    }

    pub fn enforced(rule: &str, enabled: &HashSet<&str>) -> bool {
        !OPT_IN_RULES.contains(&rule) || enabled.contains(rule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_respects_path_boundaries() {
        let prefixes = vec!["crates".to_string(), "crates/lints/".to_string()];
        assert!(Args::selected("crates/deuce/src/card.rs", &prefixes));
        assert!(Args::selected("crates", &prefixes));
        assert!(Args::selected("crates/lints/src/lib.rs", &prefixes));
        assert!(!Args::selected("crates-old/a.rs", &prefixes));
        assert!(Args::selected("anything/else.rs", &[]));
    }

    #[test]
    fn an_opt_in_rule_gates_nothing() {
        let none = HashSet::new();
        let asked = HashSet::from(["forwarding-closure"]);
        assert!(!Args::enforced("forwarding-closure", &none));
        assert!(Args::enforced("forwarding-closure", &asked));
    }

    #[test]
    fn every_other_rule_gates_every_file_a_pr_touches() {
        let none = HashSet::new();
        RULES
            .iter()
            .filter(|rule| !OPT_IN_RULES.contains(rule))
            .for_each(|rule| assert!(Args::enforced(rule, &none)));
    }
}
