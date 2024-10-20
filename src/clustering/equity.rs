use super::abstraction::Abstraction;
use super::histogram::Histogram;
use super::metric::Metric;
use crate::transport::coupling::Coupling;
use crate::transport::measure::Measure;
use crate::transport::support::Support;

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

impl Support for i8 {}

impl Measure for Equity {
    type X = i8; // Abstraction::Equity(i8) variant
    type Y = i8; // Abstraction::Equity(i8) variant
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32 {
        (x - y).abs() as f32
    }
}

impl Coupling for Equity {
    type M = Metric;
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
impl Equity {
    pub fn variation(x: &Histogram, y: &Histogram) -> f32 {
        // assert!(matches!(x.peek(), Abstraction::Equity(_)));
        // assert!(matches!(y.peek(), Abstraction::Equity(_)));
        let mut total = 0.;
        let mut cdf_x = 0.;
        let mut cdf_y = 0.;
        for abstraction in Abstraction::range() {
            cdf_x += x.weight(abstraction);
            cdf_y += y.weight(abstraction);
            total += (cdf_x - cdf_y).abs();
        }
        total / 2.
    }

    #[allow(dead_code)]
    pub fn euclidean(x: &Histogram, y: &Histogram) -> f32 {
        // assert!(matches!(x.peek(), Abstraction::Equity(_)));
        // assert!(matches!(y.peek(), Abstraction::Equity(_)));
        let mut total = 0.;
        for abstraction in Abstraction::range() {
            let x_density = x.weight(abstraction);
            let y_density = y.weight(abstraction);
            let delta = x_density - y_density;
            total += delta * delta;
        }
        total.sqrt()
    }

    #[allow(dead_code)]
    pub fn chisq(x: &Histogram, y: &Histogram) -> f32 {
        // assert!(matches!(x.peek(), Abstraction::Equity(_)));
        // assert!(matches!(y.peek(), Abstraction::Equity(_)));
        let mut total = 0.;
        for abstraction in Abstraction::range() {
            let x_density = x.weight(abstraction);
            let y_density = y.weight(abstraction);
            let delta = x_density - y_density;
            total += delta * delta / (x_density + y_density);
        }
        total
    }
}
