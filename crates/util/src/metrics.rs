use crate::*;
use serde::{Deserialize, Serialize};

/// Aggregate statistics over N played hands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Summary {
    #[serde(default)]
    pub population: usize,
    pub hands: usize,
    pub won: Chips,
    pub vpip: usize,
    pub pfr: usize,
    pub showdowns: usize,
    pub showdown_wins: usize,
    pub folds: usize,
    pub calls: usize,
    pub raises: usize,
    pub checks: usize,
    pub total_pot: i64,
    pub series: Vec<Chips>,
}

impl Summary {
    pub fn bb_per_hand(&self) -> Utility {
        ratio(self.won as f32, self.hands as f32) / B_BLIND as f32
    }

    pub fn mbb_per_hand(&self) -> Utility {
        self.bb_per_hand() * 1000.0
    }

    pub fn vpip_pct(&self) -> Probability {
        ratio(self.vpip as f32, self.hands as f32)
    }

    pub fn pfr_pct(&self) -> Probability {
        ratio(self.pfr as f32, self.hands as f32)
    }

    pub fn wtsd_pct(&self) -> Probability {
        ratio(self.showdowns as f32, self.hands as f32)
    }

    pub fn wsd_pct(&self) -> Probability {
        ratio(self.showdown_wins as f32, self.showdowns as f32)
    }

    pub fn aggression(&self) -> f32 {
        ratio(self.raises as f32, self.calls as f32)
    }

    pub fn avg_pot(&self) -> f32 {
        ratio(self.total_pot as f32, self.hands as f32)
    }

    pub fn cumulative(&self) -> Vec<Chips> {
        self.series
            .iter()
            .scan(0i16, |acc, &x| {
                *acc = acc.saturating_add(x);
                Some(*acc)
            })
            .collect()
    }

    pub fn stddev(&self) -> f32 {
        let mean = ratio(self.won as f32, self.hands as f32);
        self.series
            .iter()
            .map(|&x| (x as f32 - mean) * (x as f32 - mean))
            .sum::<f32>()
            .then(|ss| ratio(ss, self.hands as f32))
            .sqrt()
    }

    pub fn stderr(&self) -> f32 {
        ratio(self.stddev(), (self.hands as f32).sqrt())
    }
}

/// AIVAT-adjusted result for a single hand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AivatResult {
    pub raw: Chips,
    pub adjusted: Utility,
    pub corrections: Utility,
    pub hero_corrections: Utility,
    pub villain_corrections: Utility,
    pub chance_corrections: Utility,
}

/// AIVAT-adjusted aggregate: lean wire type with no embedded Summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AivatDelta {
    pub series: Vec<Utility>,
    pub won: Utility,
    pub stderr: Utility,
    pub reduction: f32,
    pub pvalue: f32,
}

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

/// Extension trait for f32 to enable method chaining.
trait ThenF32 {
    fn then(self, f: impl FnOnce(f32) -> f32) -> f32;
}
impl ThenF32 for f32 {
    fn then(self, f: impl FnOnce(f32) -> f32) -> f32 {
        f(self)
    }
}
