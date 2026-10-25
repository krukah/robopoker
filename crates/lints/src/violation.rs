#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    line: usize,
    end_line: usize,
    rule: &'static str,
    message: String,
}

impl Violation {
    pub fn new(line: usize, end_line: usize, rule: &'static str, message: String) -> Self {
        Self {
            line,
            end_line,
            rule,
            message,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn end_line(&self) -> usize {
        self.end_line
    }

    pub fn rule(&self) -> &'static str {
        self.rule
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// The message as a GitHub Actions workflow command carries it.
    pub fn escaped(&self) -> String {
        self.message()
            .replace('%', "%25")
            .replace('\r', "%0D")
            .replace('\n', "%0A")
    }
}
