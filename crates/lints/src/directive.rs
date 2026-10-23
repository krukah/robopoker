use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::*;

static ALLOW_DIRECTIVE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^lint:allow\(([^)]*)\)(.*)$").unwrap());

pub(crate) struct Directive {
    allows: HashSet<&'static str>,
    violations: Vec<Violation>,
}

impl Directive {
    pub(crate) fn new(allows: HashSet<&'static str>, violations: Vec<Violation>) -> Self {
        Self { allows, violations }
    }

    pub(crate) fn parse(block: &Block) -> Option<Self> {
        let first_line = block.content().first().map_or("", String::as_str).trim();
        let captures = ALLOW_DIRECTIVE.captures(first_line)?;
        let mut allows: HashSet<&'static str> = HashSet::new();
        let mut violations = Vec::new();
        let names = captures[1]
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        if names.is_empty() {
            violations.push(Violation::new(
                block.start_line(),
                block.start_line(),
                "invalid-allow",
                format!("lint:allow must name at least one rule; valid rules: {}", RULES.join(", ")),
            ));
        }
        for name in names {
            match RULES.iter().find(|rule| **rule == name) {
                Some(rule) => {
                    allows.insert(rule);
                }
                None => violations.push(Violation::new(
                    block.start_line(),
                    block.start_line(),
                    "invalid-allow",
                    format!("unknown rule `{name}`; valid rules: {}", RULES.join(", ")),
                )),
            }
        }
        if captures[2].trim().is_empty() {
            violations.push(Violation::new(
                block.start_line(),
                block.start_line(),
                "invalid-allow",
                "lint:allow requires a reason after the closing parenthesis".to_string(),
            ));
        }
        Some(Self::new(allows, violations))
    }

    /// Reads a `lint:allow(<rule>)` directive out of one raw comment node.
    pub(crate) fn grants(text: &str, rule: &str) -> bool {
        ALLOW_DIRECTIVE
            .captures(Self::strip(text))
            .is_some_and(|captures| captures[1].split(',').any(|name| name.trim() == rule))
    }

    /// A directive among the comments that open a file exempts the whole file.
    pub(crate) fn heads(src: &str, rule: &str) -> bool {
        src.lines()
            .take_while(|line| line.trim_start().starts_with("//"))
            .any(|line| Self::grants(line.trim(), rule))
    }

    fn strip(text: &str) -> &str {
        text.trim_start()
            .trim_start_matches("/*")
            .trim_start_matches("//")
            .trim_start_matches(['/', '!', '*'])
            .trim()
    }

    pub(crate) fn allows(&self) -> &HashSet<&'static str> {
        &self.allows
    }

    pub(crate) fn violations(&self) -> &[Violation] {
        &self.violations
    }
}
