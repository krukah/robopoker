use super::*;
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
    pub fn population(&self) -> usize {
        self.population
    }

    pub fn hands(&self) -> usize {
        self.hands
    }

    pub fn won(&self) -> Chips {
        self.won
    }

    pub fn vpip(&self) -> usize {
        self.vpip
    }

    pub fn pfr(&self) -> usize {
        self.pfr
    }

    pub fn showdowns(&self) -> usize {
        self.showdowns
    }

    pub fn showdown_wins(&self) -> usize {
        self.showdown_wins
    }

    pub fn folds(&self) -> usize {
        self.folds
    }

    pub fn calls(&self) -> usize {
        self.calls
    }

    pub fn raises(&self) -> usize {
        self.raises
    }

    pub fn checks(&self) -> usize {
        self.checks
    }

    pub fn total_pot(&self) -> i64 {
        self.total_pot
    }

    pub fn series(&self) -> &[Chips] {
        &self.series
    }
}

impl Summary {
    pub fn bb_per_hand(&self) -> Utility {
        Numeric::ratio(self.won() as f32, self.hands() as f32) / B_BLIND as f32
    }

    pub fn mbb_per_hand(&self) -> Utility {
        self.bb_per_hand() * 1000.0
    }

    pub fn vpip_pct(&self) -> Probability {
        Numeric::ratio(self.vpip() as f32, self.hands() as f32)
    }

    pub fn pfr_pct(&self) -> Probability {
        Numeric::ratio(self.pfr() as f32, self.hands() as f32)
    }

    pub fn wtsd_pct(&self) -> Probability {
        Numeric::ratio(self.showdowns() as f32, self.hands() as f32)
    }

    pub fn wsd_pct(&self) -> Probability {
        Numeric::ratio(self.showdown_wins() as f32, self.showdowns() as f32)
    }

    pub fn aggression(&self) -> f32 {
        Numeric::ratio(self.raises() as f32, self.calls() as f32)
    }

    pub fn avg_pot(&self) -> f32 {
        Numeric::ratio(self.total_pot() as f32, self.hands() as f32)
    }

    pub fn cumulative(&self) -> Vec<Chips> {
        self.series()
            .iter()
            .scan(0i16, |acc, &x| {
                *acc = acc.saturating_add(x);
                Some(*acc)
            })
            .collect()
    }

    pub fn stddev(&self) -> f32 {
        let mean = Numeric::ratio(self.won() as f32, self.hands() as f32);
        self.series()
            .iter()
            .map(|&x| (x as f32 - mean) * (x as f32 - mean))
            .sum::<f32>()
            .then(|ss| Numeric::ratio(ss, self.hands() as f32))
            .sqrt()
    }

    pub fn stderr(&self) -> f32 {
        Numeric::ratio(self.stddev(), (self.hands() as f32).sqrt())
    }
}

impl ThenF32 for f32 {
    fn then(self, f: impl FnOnce(f32) -> f32) -> f32 {
        f(self)
    }
}
