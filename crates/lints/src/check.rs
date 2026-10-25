use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::*;

static TICKET_REF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b[A-Z][A-Z0-9]{1,9}-[0-9]{1,6}\b").unwrap());
static PR_REF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^|[\s(])(#[0-9]{2,})\b").unwrap());
static URL_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"github\.com/[^\s)]+/(?:pull|issues)/[0-9]+|linear\.app/[^\s)]+").unwrap());
static TODO_MARKER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i:\b(?:todo|fixme)\b)|\b(?:HACK|XXX)\b").unwrap());
static BANNER_WRAPPED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[-=*#~_/\u{2550}\u{2500}\u{2501}]{3,}.*[-=*#~_/\u{2550}\u{2500}\u{2501}]{3,}$").unwrap()
});

const TICKET_EXEMPT_PREFIXES: &[&str] = &[
    "UTF", "ISO", "SHA", "AES", "RSA", "CRC", "IEEE", "RFC", "NIST", "HMAC", "MD", "FIPS", "PKCS", "OAUTH", "HTTP",
    "TLS", "IPV", "BASE",
];

/// Box-drawing and block characters rule a line exactly as `-` and `=` do.
const BANNER_CHARS: &str = "-=*#~_+/<>|.\u{2550}\u{2500}\u{2501}\u{2504}\u{2505}\u{2508}\u{2509}\u{254c}\u{254d}\u{2580}\u{2584}\u{2588}\u{2591}\u{2592}\u{2593}\u{2551}\u{2554}\u{2557}\u{255a}\u{255d}\u{2560}\u{2563}\u{2566}\u{2569}\u{256c}\u{250c}\u{2510}\u{2514}\u{2518}\u{2502}\u{251c}\u{2524}\u{252c}\u{2534}\u{253c}";

impl Block {
    /// The rules a one-line directive names, carried onto the block directly
    /// below it.
    pub(crate) fn carried_allows(
        index: usize,
        blocks: &[Block],
        directives: &[Option<Directive>],
    ) -> HashSet<&'static str> {
        if index == 0 {
            return HashSet::new();
        }
        let previous = &blocks[index - 1];
        let adjacent = previous.end_line() + 1 == blocks[index].start_line();
        match &directives[index - 1] {
            Some(directive) if !previous.doc() && previous.content().len() == 1 && adjacent => {
                directive.allows().clone()
            }
            _ => HashSet::new(),
        }
    }

    pub(crate) fn check(
        &self,
        directive: Option<&Directive>,
        carried: &HashSet<&'static str>,
        language: Language,
        out: &mut Vec<Violation>,
    ) {
        let mut allowed = carried.clone();
        let mut content_start = 0;
        if let Some(directive) = directive {
            content_start = 1;
            allowed.extend(directive.allows().iter().copied());
            out.extend(directive.violations().iter().cloned());
        }
        let lines = &self.content()[content_start..];
        if self.doc() {
            self.check_doc(lines, &allowed, out);
        }
        self.check_placement(lines, &allowed, out);
        self.check_prose(lines, &allowed, language, out);
    }

    fn check_doc(&self, lines: &[String], allowed: &HashSet<&'static str>, out: &mut Vec<Violation>) {
        let text_lines = lines.iter().filter(|line| !line.trim().is_empty()).count();
        let hidden = self
            .hidden()
            .filter(|(_, cap)| text_lines > *cap && !allowed.contains("hidden-doc-comment"));
        match hidden {
            Some((target, cap)) => out.push(Violation::new(self.start_line(), self.end_line(), "hidden-doc-comment", format!(
                    "{text_lines}-line doc comment on {target}, which rustdoc does not render; the maximum here is {cap}"
                ))),
            None if !allowed.contains("long-doc-comment")
                && text_lines > MAX_DOC_COMMENT_LINES =>
            {
                out.push(Violation::new(self.start_line(), self.end_line(), "long-doc-comment", format!(
                        "doc comment has {text_lines} content lines; the maximum is {MAX_DOC_COMMENT_LINES}"
                    )));
            }
            None => {}
        }
    }

    fn check_placement(&self, lines: &[String], allowed: &HashSet<&'static str>, out: &mut Vec<Violation>) {
        let effective_lines = lines.len();
        if !self.doc() && !allowed.contains("long-comment") && effective_lines > MAX_COMMENT_LINES {
            out.push(Violation::new(
                self.start_line(),
                self.end_line(),
                "long-comment",
                format!("comment block is {effective_lines} lines; the maximum is {MAX_COMMENT_LINES}"),
            ));
        }
        if !self.doc()
            && !allowed.contains("type-comment")
            && let Some(context) = self.context()
        {
            out.push(Violation::new(
                self.start_line(),
                self.end_line(),
                "type-comment",
                format!("comment inside {context}; move it above the item or remove it"),
            ));
        }
    }

    fn check_prose(
        &self,
        lines: &[String],
        allowed: &HashSet<&'static str>,
        language: Language,
        out: &mut Vec<Violation>,
    ) {
        let body = lines.join("\n");
        if !allowed.contains("ticket-reference")
            && let Some(reference) = Self::find_reference(&body)
        {
            out.push(Violation::new(
                self.start_line(),
                self.end_line(),
                "ticket-reference",
                format!("comment references `{reference}`; put ticket and PR context in the commit message"),
            ));
        }
        if !allowed.contains("todo-comment")
            && let Some(marker) = TODO_MARKER.find(&body)
        {
            out.push(Violation::new(
                self.start_line(),
                self.end_line(),
                "todo-comment",
                format!("comment contains `{}`; track the work in `docs/todo.md`", marker.as_str()),
            ));
        }
        if !allowed.contains("banner-comment") && lines.iter().any(|line| Self::is_banner_line(line)) {
            out.push(Violation::new(
                self.start_line(),
                self.end_line(),
                "banner-comment",
                "decorative banner comment; remove it".to_string(),
            ));
        }
        if !self.doc() && !allowed.contains("commented-code") && language.looks_like_code(&body) {
            out.push(Violation::new(
                self.start_line(),
                self.end_line(),
                "commented-code",
                "comment is commented-out code; delete it, git history keeps it".to_string(),
            ));
        }
    }

    fn find_reference(body: &str) -> Option<String> {
        if let Some(found) = TICKET_REF.find_iter(body).find(|found| {
            let prefix = found.as_str().split('-').next().unwrap_or("");
            !TICKET_EXEMPT_PREFIXES.contains(&prefix)
        }) {
            return Some(found.as_str().to_string());
        }
        if let Some(captures) = PR_REF.captures(body) {
            return Some(captures[1].to_string());
        }
        URL_REF.find(body).map(|found| found.as_str().to_string())
    }

    fn is_banner_line(line: &str) -> bool {
        let trimmed = line.trim();
        if Self::is_table_separator(trimmed) {
            return false;
        }
        if trimmed.len() >= 4 && trimmed.chars().all(|c| BANNER_CHARS.contains(c)) {
            return true;
        }
        BANNER_WRAPPED.is_match(trimmed)
    }

    /// A markdown table separator — `|---|:-:|` — is table structure that rustdoc
    /// renders, not decoration. `|` is a banner character, so the row would
    /// otherwise read as one.
    fn is_table_separator(line: &str) -> bool {
        line.contains('|')
            && line.trim_matches('|').split('|').all(|cell| {
                let cell = cell.trim();
                !cell.is_empty() && cell.chars().all(|c| c == '-' || c == ':')
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_drawing_rule_is_a_banner() {
        assert!(Block::is_banner_line("\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}"));
        assert!(Block::is_banner_line("\u{2500}\u{2500}\u{2500}\u{2500} Section \u{2500}\u{2500}\u{2500}\u{2500}"));
        assert!(!Block::is_banner_line("a \u{2500} b"));
    }

    #[test]
    fn a_markdown_table_separator_is_not_a_banner() {
        assert!(!Block::is_banner_line("|---|---|---|"));
        assert!(!Block::is_banner_line("| -------- | :----: | ---: |"));
        assert!(!Block::is_banner_line("|---|"));
        assert!(Block::is_banner_line("-----------------"));
        assert!(Block::is_banner_line("==== Section ===="));
        assert!(Block::is_banner_line("====|===="));
        assert!(!Block::is_banner_line("| a | b |"));
    }
}
