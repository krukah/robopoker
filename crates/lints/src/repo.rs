use std::collections::HashSet;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{Context, Result, bail};
use lints::*;

pub type Target = (String, Option<Vec<Range<usize>>>);

const PATHSPECS: &[&str] = &["*.rs", "*.ts", "*.tsx"];

/// A rule that does not apply under a path. `free-function` skips `benches`,
/// where `criterion_group!` takes a path to a bare `fn`.
const SCOPED_OUT: &[(&str, &[&str])] = &[(shape::FREE_FUNCTION, &["benches"])];

/// The checkout the linter reads, and the layout it knows about.
pub struct Repo {
    root: PathBuf,
}

impl Repo {
    pub fn open() -> Result<Self> {
        Ok(Self {
            root: PathBuf::from(Self::git(None, &["rev-parse", "--show-toplevel"])?.trim()),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn changed_files(&self, base: Option<&str>) -> Result<Vec<Target>> {
        let base = self.resolve_base(base)?;
        let merge_base = Self::git(Some(self.root()), &["merge-base", &base, "HEAD"])
            .map(|out| out.trim().to_string())
            .unwrap_or(base);
        let mut diff_args = vec![
            "diff",
            merge_base.as_str(),
            "--unified=0",
            "--no-color",
            "--diff-filter=ACMR",
            "--",
        ];
        diff_args.extend(PATHSPECS);
        let diff = Self::git(Some(self.root()), &diff_args)?;
        let created = self.created_files(&merge_base)?;
        let mut targets = Self::merge_targets(Diff::parse(&diff).files(), &created);
        let mut untracked_args = vec!["ls-files", "--others", "--exclude-standard", "--"];
        untracked_args.extend(PATHSPECS);
        let untracked = Self::git(Some(self.root()), &untracked_args)?;
        targets.extend(untracked.lines().map(|path| (path.to_string(), None)));
        Ok(targets)
    }

    /// A path the PR created: added, copied, or renamed into place. A rename
    /// carries a file to a new name, and the PR answers for what arrives there —
    /// promoting `foo.rs` to `foo/mod.rs` is how logic reaches a wiring file.
    fn created_files(&self, merge_base: &str) -> Result<HashSet<String>> {
        let mut args = vec![
            "diff",
            merge_base,
            "--name-only",
            "--no-color",
            "--diff-filter=ACR",
            "--",
        ];
        args.extend(PATHSPECS);
        Ok(Self::git(Some(self.root()), &args)?
            .lines()
            .map(str::to_string)
            .collect())
    }

    /// A rename with no edit produces no hunk, so the diff alone never names the
    /// file. Every created path is a target, whether or not an edit rode along.
    fn merge_targets(parsed: Vec<(String, Vec<Range<usize>>)>, created: &HashSet<String>) -> Vec<Target> {
        let mut targets = parsed
            .into_iter()
            .map(|(path, ranges)| if created.contains(&path) { (path, None) } else { (path, Some(ranges)) })
            .collect::<Vec<Target>>();
        let mut moved = created
            .iter()
            .filter(|path| !targets.iter().any(|(seen, _)| seen == *path))
            .collect::<Vec<&String>>();
        moved.sort();
        targets.extend(moved.into_iter().map(|path| (path.clone(), None)));
        targets
    }

    fn resolve_base(&self, requested: Option<&str>) -> Result<String> {
        if let Some(base) = requested {
            return Ok(base.to_string());
        }
        for candidate in ["origin/main", "main"] {
            if Self::git(Some(self.root()), &["rev-parse", "--verify", "--quiet", &format!("{candidate}^{{commit}}")])
                .is_ok()
            {
                return Ok(candidate.to_string());
            }
        }
        bail!("neither origin/main nor main resolves; pass --base <ref>")
    }

    pub fn tracked_files(&self) -> Result<Vec<String>> {
        let mut args = vec!["ls-files", "--"];
        args.extend(PATHSPECS);
        Ok(Self::git(Some(self.root()), &args)?
            .lines()
            .map(str::to_string)
            .collect())
    }

    pub fn excluded(path: &str) -> bool {
        path.contains("/generated/")
            || path.starts_with("target/")
            || path.contains("/target/")
            || path.contains("node_modules/")
            || path.starts_with("docs/")
            || path.starts_with("dist/")
            || path.starts_with(".claude/")
    }

    pub fn scoped_out(rule: &str, path: &str) -> bool {
        SCOPED_OUT
            .iter()
            .filter(|(scoped, _)| *scoped == rule)
            .flat_map(|(_, prefixes)| prefixes.iter())
            .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}/")))
    }

    fn git(root: Option<&Path>, args: &[&str]) -> Result<String> {
        let mut command = Command::new("git");
        if let Some(root) = root {
            command.arg("-C").arg(root);
        }
        let Output { status, stdout, stderr } = command
            .args(args)
            .output()
            .with_context(|| format!("run git {args:?}"))?;
        if !status.success() {
            bail!("git {:?} failed: {}", args, String::from_utf8_lossy(&stderr).trim());
        }
        Ok(String::from_utf8_lossy(&stdout).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rule_skips_only_its_own_prefixes() {
        assert!(Repo::scoped_out("free-function", "benches/benchmarks.rs"));
        assert!(!Repo::scoped_out("one-type-per-file", "benches/benchmarks.rs"));
        assert!(!Repo::scoped_out("free-function", "crates/deuce/src/card.rs"));
        assert!(!Repo::scoped_out("free-function", "benchesque/a.rs"));
    }

    #[test]
    fn a_renamed_file_answers_as_created() {
        let created = HashSet::from(["a/mod.rs".to_string(), "b/mod.rs".to_string(), "c.rs".to_string()]);
        let parsed = vec![
            ("a/mod.rs".to_string(), Vec::from([1..2, 7..8])),
            ("d.rs".to_string(), Vec::from([3..4, 9..10])),
        ];
        let targets = Repo::merge_targets(parsed, &created);
        let whole = targets
            .iter()
            .map(|(path, ranges)| (path.as_str(), ranges.is_none()))
            .collect::<Vec<(&str, bool)>>();
        assert_eq!(whole, vec![("a/mod.rs", true), ("d.rs", false), ("b/mod.rs", true), ("c.rs", true)]);
    }

    #[test]
    fn a_created_file_is_named_once() {
        let created = HashSet::from(["a.rs".to_string()]);
        let targets = Repo::merge_targets(vec![("a.rs".to_string(), Vec::from([1..2, 7..8]))], &created);
        assert_eq!(targets.len(), 1);
        assert!(targets[0].1.is_none());
    }

    #[test]
    fn a_created_file_answers_for_every_line() {
        let created = HashSet::from(["a.rs".to_string()]);
        let targets = Repo::merge_targets(vec![("a.rs".to_string(), Vec::from([1..2, 7..8]))], &created);
        assert!(targets[0].1.is_none());
    }
}
