use super::*;
use pokerkit::Utility;

/// The light register as concrete hex — `input.css` `:root` and the
/// `studio.paper*` tailwind colours: a cream ground, hairline rules, and the
/// muted printed inks. The default look of the standalone SVG/GIF assets.
pub struct Paper;

impl Paper {
    /// Categorical cycle for games outside the poker vocabulary.
    pub const CYCLE: [&str; 4] = ["#2f6b4f", "#2a5d8f", "#b4601f", "#a8322c"];

    /// Regret blend anchors: fold grey, raise ochre for gain, call blue for loss.
    pub const BLEND: [(u32, u32, u32); 3] = [(111, 106, 99), (180, 96, 31), (42, 93, 143)];
}

impl Palette for Paper {
    fn bg(&self) -> String {
        "#eeebe4".into()
    }

    fn fg(&self) -> String {
        "#16181a".into()
    }

    fn muted(&self) -> String {
        "#6d6961".into()
    }

    fn border(&self) -> String {
        "#d6d2c8".into()
    }

    fn inverse(&self) -> String {
        "#faf9f6".into()
    }

    fn ground(&self) -> String {
        "#e6e2d6".into()
    }

    fn stripe(&self) -> String {
        "rgba(255, 255, 255, 0.22)".into()
    }

    fn felt(&self) -> String {
        "#2f5a4b".into()
    }

    fn pocket(&self) -> String {
        "#d6d0c1".into()
    }

    fn settle(&self) -> String {
        "#e9d5aa".into()
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
        "#946512".into()
    }

    fn observed(&self) -> String {
        "#a8322c".into()
    }

    fn range(&self) -> String {
        "#7a4a12".into()
    }

    fn action(&self, deed: Deed) -> String {
        match deed {
            Deed::Draw => self.felt(),
            Deed::Fold => "#6f6a63".into(),
            Deed::Check => "#2f6b4f".into(),
            Deed::Call => "#2a5d8f".into(),
            Deed::Open => "#b4601f".into(),
            Deed::Raise(ratio) => Paint::grade(ratio),
            Deed::Shove => "#a8322c".into(),
            Deed::Nth(i) => Self::CYCLE[i % Self::CYCLE.len()].into(),
        }
    }

    fn regret(&self, gain: Utility) -> String {
        let tint = if gain >= 0.0 { Self::BLEND[1] } else { Self::BLEND[2] };
        Paint::blend(Self::BLEND[0], tint, FLOOR + (1.0 - FLOOR) * f64::from(gain.abs().min(1.0)))
    }
}
