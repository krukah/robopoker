//! The feature vocabulary's five groups, as one thing instead of three.
use crate::*;
use pokerkit::*;

/// A named slice of the [`Feature`] vocabulary.
///
/// A description section needs three things that must agree: a heading, the
/// features under it, and whether its shares are of the sample or of the
/// group. Kicker quality is the odd one — "weak kicker 62%" means 62% of the
/// hands that *had* a kicker, not of the bucket — and holding that convention
/// in the caller is how a heading ends up over the wrong denominator.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Group {
    Made,
    Kickers,
    Draws,
    Shape,
    Board,
}

impl Group {
    /// Every group, in the order a description reads them.
    pub const ALL: [Self; 5] = [Self::Made, Self::Kickers, Self::Draws, Self::Shape, Self::Board];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Made => "Made",
            Self::Kickers => "Kickers",
            Self::Draws => "Draws",
            Self::Shape => "Shape",
            Self::Board => "Board",
        }
    }

    pub const fn features(self) -> &'static [Feature] {
        match self {
            Self::Made => &Feature::MADE,
            Self::Kickers => &Feature::KICKS,
            Self::Draws => &Feature::DRAWS,
            Self::Shape => &Feature::SHAPE,
            Self::Board => &Feature::BOARD,
        }
    }

    /// The group's features above a floor, most common first, each with the
    /// share this group is read in.
    pub fn shares(self, profile: &Profile, floor: Probability) -> Vec<(Feature, Probability)> {
        match self {
            Self::Kickers => profile.conditioned(self.features(), floor),
            _ => profile.ranked(self.features(), floor),
        }
    }
}
