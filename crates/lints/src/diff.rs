use std::ops::Range;

/// The lines a unified diff adds, by file. A file whose hunks only delete
/// adds nothing and drops out.
pub struct Diff {
    files: Vec<(String, Vec<Range<usize>>)>,
}

impl Diff {
    pub fn parse(diff: &str) -> Self {
        let mut files: Vec<(String, Vec<Range<usize>>)> = Vec::new();
        let mut current: Option<usize> = None;
        for line in diff.lines() {
            if let Some(path) = line.strip_prefix("+++ b/") {
                files.push((path.to_string(), Vec::new()));
                current = Some(files.len() - 1);
            } else if line.starts_with("+++ ") {
                current = None;
            } else if line.starts_with("@@")
                && let Some(index) = current
                && let Some(range) = Self::hunk(line)
            {
                files[index].1.push(range);
            }
        }
        files.retain(|(_, ranges)| !ranges.is_empty());
        Self { files }
    }

    pub fn files(self) -> Vec<(String, Vec<Range<usize>>)> {
        self.files
    }

    fn hunk(line: &str) -> Option<Range<usize>> {
        let added = line.split_whitespace().find(|token| token.starts_with('+'))?;
        let added = added.strip_prefix('+')?;
        let (start, count) = match added.split_once(',') {
            Some((start, count)) => (start.parse().ok()?, count.parse().ok()?),
            None => (added.parse().ok()?, 1),
        };
        if count == 0 {
            return None;
        }
        Some(start..start + count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_diff_extracts_added_ranges() {
        let diff = "diff --git a/rust/src/a.rs b/rust/src/a.rs\n\
                    --- a/rust/src/a.rs\n\
                    +++ b/rust/src/a.rs\n\
                    @@ -10,2 +12,3 @@ fn f() {\n\
                    +a\n+b\n+c\n\
                    @@ -20 +25 @@ fn g() {\n\
                    +d\n\
                    @@ -30,2 +40,0 @@ fn h() {\n\
                    -e\n-f\n";
        let files = Diff::parse(diff).files();
        assert_eq!(files, vec![("rust/src/a.rs".to_string(), vec![12..15, 25..26])]);
    }
}
