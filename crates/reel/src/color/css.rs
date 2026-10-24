use super::*;

/// Categorical cycle for games outside the poker vocabulary.
pub const VARS: [&str; 4] = [
    "var(--action-check)",
    "var(--action-call)",
    "var(--action-raise)",
    "var(--action-allin)",
];

/// CSS custom-property references — the paint world of the Leptos app,
/// where the theme owns the actual values.
pub struct Css;

impl Palette for Css {
    fn bg(&self) -> String {
        "var(--tm-bg)".into()
    }

    fn fg(&self) -> String {
        "var(--tm-fg)".into()
    }

    fn muted(&self) -> String {
        "var(--tm-muted)".into()
    }

    fn border(&self) -> String {
        "var(--tm-border)".into()
    }

    fn inverse(&self) -> String {
        "var(--tm-inverse)".into()
    }

    fn ground(&self) -> String {
        "var(--tree-band-a)".into()
    }

    fn stripe(&self) -> String {
        "var(--tree-stripe)".into()
    }

    fn felt(&self) -> String {
        "var(--felt)".into()
    }

    fn pocket(&self) -> String {
        "var(--street-private)".into()
    }

    fn settle(&self) -> String {
        "var(--street-settle)".into()
    }

    fn white(&self) -> String {
        "var(--card-white)".into()
    }

    fn black(&self) -> String {
        "var(--card-black)".into()
    }

    fn red(&self) -> String {
        "var(--card-red)".into()
    }

    fn gold(&self) -> String {
        "var(--accent-gold)".into()
    }

    fn observed(&self) -> String {
        "var(--hist-obs-stroke)".into()
    }

    fn range(&self) -> String {
        "var(--range-color)".into()
    }

    fn action(&self, deed: Deed) -> String {
        match deed {
            Deed::Draw => "var(--felt)".into(),
            Deed::Fold => "var(--action-fold)".into(),
            Deed::Check => "var(--action-check)".into(),
            Deed::Call => "var(--action-call)".into(),
            Deed::Open => "var(--action-raise)".into(),
            Deed::Raise(ratio) => Paint::grade(ratio),
            Deed::Shove => "var(--action-allin)".into(),
            Deed::Nth(i) => VARS[i % VARS.len()].into(),
        }
    }
}
