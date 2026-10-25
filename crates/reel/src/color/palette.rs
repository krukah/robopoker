use super::*;
use pokerkit::Utility;

/// The paint seam: one geometry, many paint worlds. [`Paper`] and [`Ink`] are
/// the site's light and dark registers as concrete hex; [`Css`] defers to the
/// app's theme through `var(--…)` references.
pub trait Palette {
    /// Canvas background.
    fn bg(&self) -> String;

    /// Foreground: committed (frozen) lines and outlines.
    fn fg(&self) -> String;

    /// Quiet labels: actors, phases, skeleton glyphs.
    fn muted(&self) -> String;

    /// Hairline rules: token faces, action cells, axes.
    fn border(&self) -> String;

    /// Knockout ink on a saturated fill: action-cell glyphs, range-cell labels.
    fn inverse(&self) -> String;

    /// The tree's own ground under every lane.
    fn ground(&self) -> String;

    /// Translucent overlay on alternate lanes, so the tree reads as a ledger.
    fn stripe(&self) -> String;

    /// Community band behind shared reveals.
    fn felt(&self) -> String;

    /// Private band behind the hero's own reveals.
    fn pocket(&self) -> String;

    /// Terminal band behind the settlement — the boundary out, painted the same rank as
    /// [`Self::felt`] and [`Self::pocket`] are the boundaries in.
    fn settle(&self) -> String;

    /// Token face.
    fn white(&self) -> String;

    /// Dark ink: black suits, glyphs on light tiles.
    fn black(&self) -> String;

    /// Red ink: red suits.
    fn red(&self) -> String;

    /// Gold ink: the abstraction series and terminal accents.
    fn gold(&self) -> String;

    /// The observation series of the equity curve.
    fn observed(&self) -> String;

    /// Range-grid ink, mixed into the ground by weight.
    fn range(&self) -> String;

    /// The action color of a deed.
    fn action(&self, deed: Deed) -> String;

    /// Diverging regret hue: warm for gain, cool for loss, saturation floored so even small
    /// updates read as colored.
    fn regret(&self, gain: Utility) -> String {
        let tint = if gain >= 0.0 { WARM } else { COOL };
        Paint::blend(NEUTRAL, tint, FLOOR + (1.0 - FLOOR) * f64::from(gain.abs().min(1.0)))
    }
}
