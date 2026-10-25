use std::collections::HashSet;

use anyhow::{Result, bail};
use clap::Parser;
use lints::*;

mod args;
mod census;
mod repo;

use args::*;
use census::*;
use repo::*;

fn main() -> Result<()> {
    let args = Args::parse();
    let repo = Repo::open()?;
    let targets: Vec<Target> = if args.all() {
        repo.tracked_files()?.into_iter().map(|path| (path, None)).collect()
    } else {
        repo.changed_files(args.base())?
    };
    if args.census() {
        let scanned = targets
            .iter()
            .map(|(path, _)| path)
            .filter(|path| !Repo::excluded(path) && Args::selected(path, args.paths()))
            .cloned()
            .collect::<Vec<_>>();
        Census::read(repo.root(), &scanned).report();
        return Ok(());
    }
    let mut results: Vec<(String, Violation)> = Vec::new();
    let mut skipped = 0usize;
    for (path, ranges) in targets {
        if Repo::excluded(&path) || !Args::selected(&path, args.paths()) {
            continue;
        }
        let Some(language) = Language::for_path(&path) else {
            continue;
        };
        let Ok(src) = std::fs::read_to_string(repo.root().join(&path)) else {
            skipped += 1;
            continue;
        };
        results.extend(
            language
                .lint_file(&path, &src, ranges.as_deref())
                .into_iter()
                .map(|violation| (path.clone(), violation)),
        );
        if let Some(violation) = Violation::crowding(repo.root(), &path, &src) {
            results.push((path.clone(), violation));
        }
    }
    let known = RULES;
    for rule in args.enable().iter().chain(args.only()) {
        if !known.contains(&rule.as_str()) {
            bail!("unknown rule `{rule}`; valid rules: {}", known.join(", "));
        }
    }
    let only = args.only().iter().map(String::as_str).collect::<HashSet<&str>>();
    let enabled = args
        .enable()
        .iter()
        .map(String::as_str)
        .chain(only.iter().copied())
        .collect::<HashSet<&str>>();
    results.retain(|(path, violation)| !Repo::scoped_out(violation.rule(), path));
    results.retain(|(_, violation)| Args::enforced(violation.rule(), &enabled));
    results.retain(|(_, violation)| only.is_empty() || only.contains(violation.rule()));
    results.sort_by(|(a_path, a), (b_path, b)| (a_path, a.line(), a.rule()).cmp(&(b_path, b.line(), b.rule())));
    #[allow(clippy::disallowed_methods)]
    let annotate = std::env::var_os("GITHUB_ACTIONS").is_some();
    for (path, violation) in &results {
        println!("{path}:{}: {}: {}", violation.line(), violation.rule(), violation.message());
        if annotate {
            println!(
                "::error file={path},line={},endLine={},title=lints({})::{}",
                violation.line(),
                violation.end_line(),
                violation.rule(),
                violation.escaped()
            );
        }
    }
    if skipped > 0 {
        eprintln!("lints: skipped {skipped} unreadable file(s)");
    }
    if results.is_empty() {
        println!("lints: no violations");
        return Ok(());
    }
    let files = results.iter().map(|(path, _)| path).collect::<HashSet<_>>();
    println!();
    println!("lints: {} violation(s) in {} file(s)", results.len(), files.len());
    println!("If a comment must stay as written, make its first line: // lint:allow(<rule>) <reason>");
    std::process::exit(1);
}
