//! Result of one hand against an external opponent.

use kicker::Turn;

/// What we took off the table, from hero's seat.
///
/// Both figures are in big blinds so they compare across services regardless of each one's chip
/// denomination. `aivat` is the *service's* luck-adjusted estimate of the same hand — present
/// only where the opponent computes one (Slumbot does not), which is why it reads as a second
/// opinion rather than a replacement.
#[derive(Debug, Clone, Copy)]
pub struct Outcome {
    hero: Turn,
    chips: f64,
    aivat: Option<f64>,
}

impl Outcome {
    /// A hand scored only by the chips that moved.
    pub fn new(hero: Turn, chips: f64) -> Self {
        Self {
            hero,
            chips,
            aivat: None,
        }
    }

    /// Attach the opponent's luck-adjusted score for this hand, in bb.
    pub fn with_aivat(self, aivat: Option<f64>) -> Self {
        Self { aivat, ..self }
    }

    /// Which seat we held.
    pub fn hero(&self) -> Turn {
        self.hero
    }

    /// Chips won (negative: lost) this hand, in bb.
    pub fn chips(&self) -> f64 {
        self.chips
    }

    /// The opponent's AIVAT estimate for this hand, in bb, if it reports one.
    pub fn aivat(&self) -> Option<f64> {
        self.aivat
    }
}
