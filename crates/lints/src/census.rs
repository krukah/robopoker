use std::path::Path;

use lints::Language;
use lints::complexity::{MAX_COMPLEXITY, Unit};

/// Every unit found in a scan, ranked worst first.
pub struct Census(Vec<(String, Unit)>);

impl Census {
    pub fn read(root: &Path, paths: &[String]) -> Self {
        let mut scored = paths
            .iter()
            .filter_map(|path| Language::for_path(path).map(|language| (path, language)))
            .filter_map(|(path, language)| Self::score(root, path, language))
            .flatten()
            .collect::<Vec<_>>();
        scored.sort_by(Self::worst_first);
        Self(scored)
    }

    pub fn report(&self) {
        self.0.iter().for_each(Self::line);
        println!();
        println!("{}", self.summary());
    }

    fn score(root: &Path, path: &str, language: Language) -> Option<Vec<(String, Unit)>> {
        std::fs::read_to_string(root.join(path))
            .ok()
            .map(|src| language.units(&src))
            .map(|units| units.into_iter().map(|unit| (path.to_string(), unit)).collect())
    }

    fn line(entry: &(String, Unit)) {
        let (path, unit) = entry;
        println!("{:>4}  {path}:{}: {}", unit.score(), unit.line(), unit.name());
    }

    fn summary(&self) -> String {
        let ranked = self.ranked();
        let over = ranked.iter().filter(|score| **score > MAX_COMPLEXITY).count();
        format!(
            "{} function(s); p50 {} p90 {} p95 {} p99 {} max {}\n{over} over the threshold of {MAX_COMPLEXITY} ({:.2}%)",
            ranked.len(),
            Self::quantile(&ranked, 0.50),
            Self::quantile(&ranked, 0.90),
            Self::quantile(&ranked, 0.95),
            Self::quantile(&ranked, 0.99),
            Self::quantile(&ranked, 1.00),
            100.0 * over as f64 / ranked.len().max(1) as f64,
        )
    }

    /// Scores in ascending order, which is what a quantile indexes into.
    fn ranked(&self) -> Vec<usize> {
        self.0.iter().rev().map(|(_, unit)| unit.score()).collect()
    }

    fn quantile(ranked: &[usize], quantile: f64) -> usize {
        ranked
            .len()
            .checked_sub(1)
            .map(|last| (last as f64 * quantile) as usize)
            .and_then(|index| ranked.get(index))
            .copied()
            .unwrap_or_default()
    }

    fn worst_first(left: &(String, Unit), right: &(String, Unit)) -> std::cmp::Ordering {
        right
            .1
            .score()
            .cmp(&left.1.score())
            .then_with(|| (&left.0, left.1.line()).cmp(&(&right.0, right.1.line())))
    }
}
