use super::*;

/// Categorical cycle for games outside the poker vocabulary.
pub const CYCLE: [&str; 4] = ["#34d399", "#60a5fa", "#fb923c", "#f87171"];

/// Concrete dark-mode hex, mirroring `input.css` `.dark` values — the paint
/// world of the standalone SVG/GIF assets.
pub struct Hex;

impl Palette for Hex {
    fn bg(&self) -> String {
        "#121212".into()
    }

    fn fg(&self) -> String {
        "#f5f5f5".into()
    }

    fn muted(&self) -> String {
        "#9a9a9a".into()
    }

    fn border(&self) -> String {
        "#505050".into()
    }

    fn felt(&self) -> String {
        "#0d4027".into()
    }

    fn pocket(&self) -> String {
        "#24242c".into()
    }

    fn settle(&self) -> String {
        "#3a2c0e".into()
    }

    fn white(&self) -> String {
        "#ffffff".into()
    }

    fn black(&self) -> String {
        "#000000".into()
    }

    fn red(&self) -> String {
        "#f87171".into()
    }

    fn gold(&self) -> String {
        "#fbbf24".into()
    }

    fn action(&self, deed: Deed) -> String {
        match deed {
            Deed::Draw => "#0d4027".into(),
            Deed::Fold => "#6b6b6b".into(),
            Deed::Check => "#34d399".into(),
            Deed::Call => "#60a5fa".into(),
            Deed::Open => "#fb923c".into(),
            Deed::Raise(ratio) => Paint::grade(ratio),
            Deed::Shove => "#f87171".into(),
            Deed::Nth(i) => CYCLE[i % CYCLE.len()].into(),
        }
    }
}
