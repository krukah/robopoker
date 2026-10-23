/// Font stack for tabular content (glyphs, labels, sizes).
pub const MONO: &str = "ui-monospace, 'IBM Plex Mono', 'SFMono-Regular', Menlo, monospace";

/// Font stack for card suit glyphs.
pub const SERIF: &str = "'IBM Plex Serif', Georgia, serif";

/// Semantic font role, resolved to a stack at render time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Mono,
    Serif,
}

impl Face {
    pub(crate) fn svg(&self) -> &'static str {
        match self {
            Self::Mono => MONO,
            Self::Serif => SERIF,
        }
    }
}
