use super::*;
use kicker::*;
use monge::*;
use pokerkit::*;

/// Distance metric for river equity distributions, whose abstractions are raw
/// showdown equities in `[0, 1]`.
///
/// Over a 1D interval EMD equals the L1 distance between CDFs (total
/// variation), so river distances skip Sinkhorn entirely.
pub struct Equity;

impl Measure for Equity {
    type X = ClusterAbs;
    type Y = ClusterAbs;

    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32 {
        (Probability::from(**x) - Probability::from(**y)).abs()
    }
}

/// Distance metrics for equity histograms, exploiting the 1D structure of
/// `[0,1]`-valued distributions instead of general optimal transport.
#[allow(dead_code)]
impl Equity {
    /// Total variation (L1 between CDFs) — equals EMD in 1D under `|x-y|`.
    pub fn variation(x: &Histogram, y: &Histogram) -> Energy {
        let mut cdf_x = 0.0;
        let mut cdf_y = 0.0;
        Abstraction::range()
            .map(|abstraction| {
                cdf_x += x.density(&abstraction);
                cdf_y += y.density(&abstraction);
                cdf_x - cdf_y
            })
            .map(f32::abs)
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
