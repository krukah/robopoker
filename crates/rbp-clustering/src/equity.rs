use super::*;
use rbp_core::*;
use rbp_gameplay::*;
use rbp_transport::*;

/// Distance metric for river equity distributions.
///
/// River abstractions represent raw showdown equity values in [0, 1].
/// This struct provides distance measures between equity abstractions
/// and between histograms over equity values.
///
/// # EMD on [0, 1]
///
/// For distributions over a 1D interval, EMD equals the L1 distance between
/// CDFs (total variation). This avoids the computational cost of Sinkhorn
/// optimal transport used for higher-dimensional abstraction spaces.
///
/// # Kontorovich-Rubinstein Dual
///
/// The ground distance `|x - y|` between equity values makes the coupling
/// constraint equivalent to doubly-stochastic marginals on [0,1] × [0,1].
pub struct Equity;

impl Measure for Equity {
    type X = Abstraction; //::Equity(i8) variant
    type Y = Abstraction; //::Equity(i8) variant
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32 {
        (Probability::from(*x) - Probability::from(*y)).abs()
    }
}

/// Distance metrics for equity histograms.
///
/// These exploit the 1D structure of [0,1]-valued distributions to provide
/// efficient alternatives to general optimal transport.
#[allow(dead_code)]
impl Equity {
    /// Total variation distance (L1 between CDFs).
    /// This equals EMD for 1D distributions with |x-y| ground cost.
    pub fn variation(x: &Histogram, y: &Histogram) -> Energy {
        let mut cdf_x = 0.0;
        let mut cdf_y = 0.0;
        Abstraction::range()
            .map(|abstraction| {
                cdf_x += x.density(&abstraction);
                cdf_y += y.density(&abstraction);
                cdf_x - cdf_y
            })
            .map(|delta| delta.abs())
            .sum::<Energy>()
            / Abstraction::size() as Energy
    }
    /// Euclidean (L2) distance between PMF vectors.
    pub fn euclidean(x: &Histogram, y: &Histogram) -> Energy {
        Abstraction::range()
            .map(|abstraction| x.density(&abstraction) - y.density(&abstraction))
            .map(|delta| delta * delta)
            .sum::<Energy>()
            .sqrt()
    }
    /// Chi-square divergence (asymmetric).
    pub fn chisquare(x: &Histogram, y: &Histogram) -> Energy {
        Abstraction::range()
            .map(|abstraction| (x.density(&abstraction), y.density(&abstraction)))
            .map(|(x, y)| (x - y).powi(2) / (x + y))
            .sum::<Energy>()
    }
    /// Total variation distance (L1 between PMFs, not CDFs).
    pub fn divergent(x: &Histogram, y: &Histogram) -> Energy {
        Abstraction::range()
            .map(|abstraction| (x.density(&abstraction), y.density(&abstraction)))
            .map(|(x, y)| (x - y).abs())
            .sum::<Energy>()
    }
}
