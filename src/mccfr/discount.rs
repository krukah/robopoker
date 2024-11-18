use crate::Utility;

#[derive(Debug)]
pub struct Discount {
    period: usize, // interval between strategy updates.
    alpha: f32,    // α parameter. controls recency bias.
    omega: f32,    // ω parameter. controls recency bias.
    gamma: f32,    // γ parameter. controls recency bias.
}

impl Discount {
    pub const fn default() -> &'static Self {
        &Self {
            period: 1,
            alpha: 1.5,
            omega: 0.5,
            gamma: 2.0,
        }
    }

    pub fn policy(&self, t: usize) -> f32 {
        (t as f32 / (t as f32 + 1.)).powf(self.gamma)
    }

    pub fn regret(&self, t: usize, regret: Utility) -> Utility {
        if t % self.period != 0 {
            1.
        } else if regret > 0. {
            let x = (t as f32 / self.period as f32).powf(self.alpha);
            x / (x + 1.)
        } else if regret < 0. {
            let x = (t as f32 / self.period as f32).powf(self.omega);
            x / (x + 1.)
        } else {
            1.
        }
    }
}
