use pokerkit::Probability;

/// Neutral base of the regret blend.
pub const NEUTRAL: (u32, u32, u32) = (120, 120, 120);

/// Warm regret tint — positive counterfactual gain.
pub const WARM: (u32, u32, u32) = (251, 146, 60);

/// Cool regret tint — booked loss.
pub const COOL: (u32, u32, u32) = (56, 182, 240);

/// Saturation floor of the regret blend.
pub const FLOOR: f64 = 0.5;

/// Small-size end of the raise gradient.
pub const GOLD: (u32, u32, u32) = (217, 119, 6);

/// Overbet end of the raise gradient.
pub const DANGER: (u32, u32, u32) = (220, 38, 38);

/// Pot-ratio domain of the raise gradient.
pub const RANGE: (f64, f64) = (0.25, 4.0);

/// Namespace for the paint routines.
pub(crate) struct Paint;

impl Paint {
    /// Gold-to-red gradient over pot ratio, shared verbatim with the client's `raise_color` so both
    /// worlds agree on every raise size.
    pub(crate) fn grade(ratio: Probability) -> String {
        let ratio = f64::from(ratio).clamp(RANGE.0, RANGE.1);
        Paint::blend(GOLD, DANGER, (ratio - RANGE.0) / (RANGE.1 - RANGE.0))
    }

    pub(crate) fn blend(base: (u32, u32, u32), tint: (u32, u32, u32), t: f64) -> String {
        let mix = |a: u32, b: u32| (f64::from(a) + t * (f64::from(b) - f64::from(a))) as u32;
        format!("#{:02X}{:02X}{:02X}", mix(base.0, tint.0), mix(base.1, tint.1), mix(base.2, tint.2))
    }
}
