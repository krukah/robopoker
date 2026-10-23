use super::*;
use pokerkit::Utility;

/// One geometry, two paint worlds.
pub trait Palette {
    /// Canvas background.
    fn bg(&self) -> String;

    /// Foreground: committed (frozen) lines and outlines.
    fn fg(&self) -> String;

    /// Quiet labels: actors, phases, skeleton glyphs.
    fn muted(&self) -> String;

    /// Hairline borders on token faces.
    fn border(&self) -> String;

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

    /// The action color of a deed.
    fn action(&self, deed: Deed) -> String;

    /// Diverging regret hue: warm for gain, cool for loss, saturation floored so even small
    /// updates read as colored.
    fn regret(&self, gain: Utility) -> String {
        let tint = if gain >= 0.0 { WARM } else { COOL };
        Paint::blend(NEUTRAL, tint, FLOOR + (1.0 - FLOOR) * f64::from(gain.abs().min(1.0)))
    }
}
