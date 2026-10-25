/// Numeric helpers shared by the summary statistics.
pub struct Numeric;
impl Numeric {
    pub fn ratio(num: f32, den: f32) -> f32 {
        if den > 0.0 { num / den } else { 0.0 }
    }

    /// Approximate standard normal CDF using the Abramowitz & Stegun formula.
    pub fn erf(x: f32) -> f32 {
        const A: f32 = 0.2316419;
        const B: f32 = 0.3989423;
        const C: f32 = 0.3193815;
        const D: f32 = -0.3565638;
        const E: f32 = 1.781478;
        const F: f32 = -1.821256;
        const G: f32 = 1.330274;
        let t = 1.0 / (1.0 + A * x.abs());
        let d = B * (-x * x / 2.0).exp();
        let p = d * t * (C + t * (D + t * (E + t * (F + t * G))));
        if x > 0.0 { 1.0 - p } else { p }
    }
}
