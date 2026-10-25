/// The site's `--font-text` stack: Barlow, then the platform sans.
pub const TEXT: &str = "Barlow, -apple-system, BlinkMacSystemFont, 'Helvetica Neue', Arial, sans-serif";

/// The site's `--font-data` stack: IBM Plex Mono, then the platform mono.
pub const DATA: &str = "'IBM Plex Mono', ui-monospace, 'SFMono-Regular', Menlo, monospace";

/// Tracking of the uppercase gutter captions, as `.tm-tree-gut` sets it.
pub const TRACK: &str = "0.1em";

/// Semantic font role, resolved to SVG font attributes at render time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Text,
    Caps,
    Data,
    Pip,
}

impl Face {
    pub(crate) fn svg(&self) -> String {
        match self {
            Self::Text | Self::Pip => format!(r#"font-family="{TEXT}""#),
            Self::Caps => format!(r#"font-family="{TEXT}" letter-spacing="{TRACK}""#),
            Self::Data => format!(r#"font-family="{DATA}""#),
        }
    }
}
