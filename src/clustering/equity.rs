use super::abstraction::Abstraction;
use super::histogram::Histogram;
use crate::transport::coupling::Coupling;
use crate::transport::density::Density;
use crate::transport::measure::Measure;

/// useful struct for methods that help in calculating
/// optimal transport between two Equity Histograms.
/// more broadly, this can generalize to calclaute distance
/// between arbitrary distributions over the [0, 1] interval.
///
/// in the Kontorovich-Rubinstein dual formulation of optimal transport,
/// we can think of the constraint as being probabilistic unitarity.
/// equivalently, we constrain the coupling to be doubly stochastic
/// over the product of support spaces, i.e. [0, 1] x [0, 1].
pub struct Equity;

impl Measure for Equity {
    type X = Abstraction; //::Equity(i8) variant
    type Y = Abstraction; //::Equity(i8) variant
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32 {
        match (x, y) {
            (Self::X::Percent(x), Self::Y::Percent(y)) => (*x as f32 - *y as f32).abs(),
            _ => unreachable!("only equity distance for equity abstractions. perhaps Self::X should be f32 to avoid this pattern match"),
        }
    }
}

impl Coupling for Equity {
    type M = Self;
    type X = Abstraction; //::Equity(i8) variant
    type Y = Abstraction; //::Equity(i8) variant
    type P = Histogram;
    type Q = Histogram;
    /// this would just be the difference between
    /// CDF's of the two Histograms at points x and y.
    fn flow(&self, _: &Self::X, _: &Self::Y) -> f32 {
        todo!("implementation would require storage of the optimal transport plan, in which case this fn would become a simple lookup.")
    }
    /// we could use any of the (Histogram, Histogram) -> f32
    /// distance metrics defined in this module.
    /// absolute variation is a reasonable default, and it corresponds
    /// to the Wasserstein-1 distance between inverse CDFs.
    fn cost(&self, x: &Self::P, y: &Self::Q, _: &Self::M) -> f32 {
        Self::variation(x, y)
    }
}

/// different distance metrics over Equity Histograms
/// conveniently have properties of distributions over the [0, 1] interval.
#[allow(dead_code)]
impl Equity {
    pub fn variation(x: &Histogram, y: &Histogram) -> f32 {
        Abstraction::range()
            .iter()
            .map(|abstraction| (x.density(abstraction), y.density(abstraction)))
            .scan((0., 0.), |cdf, (px, py)| {
                Some({
                    cdf.0 += px;
                    cdf.1 += py;
                    *cdf
                })
            })
            .map(|(x, y)| (x - y).abs())
            .sum::<f32>()
            / Abstraction::range().len() as f32
            / 2.
    }
    pub fn euclidean(x: &Histogram, y: &Histogram) -> f32 {
        Abstraction::range()
            .iter()
            .map(|abstraction| x.density(abstraction) - y.density(abstraction))
            .map(|delta| delta * delta)
            .sum::<f32>()
            .sqrt()
    }
    pub fn chisquare(x: &Histogram, y: &Histogram) -> f32 {
        Abstraction::range()
            .iter()
            .map(|abstraction| (x.density(abstraction), y.density(abstraction)))
            .map(|(x, y)| (x - y).powi(2) / (x + y))
            .sum::<f32>()
    }
    pub fn divergent(x: &Histogram, y: &Histogram) -> f32 {
        Abstraction::range()
            .iter()
            .map(|abstraction| (x.density(abstraction), y.density(abstraction)))
            .map(|(x, y)| (x - y).abs())
            .sum::<f32>()
    }
}
