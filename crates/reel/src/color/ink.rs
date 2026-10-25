use super::*;

/// The dark register as concrete hex — `input.css` `.dark` and the
/// `studio.ink*` tailwind colours: graphite ground and the 400-tier inks.
pub struct Ink;

impl Ink {
    /// Categorical cycle for games outside the poker vocabulary.
    pub const CYCLE: [&str; 4] = ["#34d399", "#60a5fa", "#fb923c", "#f87171"];
}

impl Palette for Ink {
    fn bg(&self) -> String {
        "#0c0d0f".into()
    }

    fn fg(&self) -> String {
        "#e9eaec".into()
    }

    fn muted(&self) -> String {
        "#8b929b".into()
    }

    fn border(&self) -> String {
        "#2a2e34".into()
    }

    fn inverse(&self) -> String {
        "#0c0d0f".into()
    }

    fn ground(&self) -> String {
        "#15161a".into()
    }

    fn stripe(&self) -> String {
        "rgba(255, 255, 255, 0.045)".into()
    }

    fn felt(&self) -> String {
        "#164e51".into()
    }

    fn pocket(&self) -> String {
        "#2b2b33".into()
    }

    fn settle(&self) -> String {
        "#4a3a14".into()
    }

    fn white(&self) -> String {
        "#ffffff".into()
    }

    fn black(&self) -> String {
        "#000000".into()
    }

    fn red(&self) -> String {
        "#dc2626".into()
    }

    fn gold(&self) -> String {
        "#fbbf24".into()
    }

    fn observed(&self) -> String {
        "#f87171".into()
    }

    fn range(&self) -> String {
        "#eadcc2".into()
    }

    fn action(&self, deed: Deed) -> String {
        match deed {
            Deed::Draw => self.felt(),
            Deed::Fold => "#6f6a63".into(),
            Deed::Check => "#34d399".into(),
            Deed::Call => "#60a5fa".into(),
            Deed::Open => "#fb923c".into(),
            Deed::Raise(ratio) => Paint::grade(ratio),
            Deed::Shove => "#f87171".into(),
            Deed::Nth(i) => Self::CYCLE[i % Self::CYCLE.len()].into(),
        }
    }
}
