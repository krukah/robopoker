use super::abstraction::Abstraction;
use super::histogram::Histogram;
use crate::transport::measure::Measure;
use crate::Energy;

/// useful struct for grouping methods that help in calculating
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
            (Self::X::Percent(x), Self::Y::Percent(y)) => (*x as f32 - *y as f32).abs() / Abstraction::size() as f32,
            _ => unreachable!("should make Abstraction::distance a thing. perhaps Self::X should be f32 to avoid this pattern match"),
        }
    }
}

/// different distance metrics over Equity Histograms
/// conveniently have properties of distributions over the [0, 1] interval.
#[allow(dead_code)]
impl Equity {
    pub fn variation(x: &Histogram, y: &Histogram) -> Energy {
        Abstraction::range()
            .map(|abstraction| (x.density(&abstraction), y.density(&abstraction)))
            .scan((0., 0.), |cdf, (px, py)| {
                Some({
                    cdf.0 += px;
                    cdf.1 += py;
                    cdf.clone()
                })
            })
            .map(|(x, y)| (x - y).abs())
            .sum::<Energy>()
            / Abstraction::size() as Energy
            / 2.
    }
    pub fn euclidean(x: &Histogram, y: &Histogram) -> Energy {
        Abstraction::range()
            .map(|abstraction| x.density(&abstraction) - y.density(&abstraction))
            .map(|delta| delta * delta)
            .sum::<Energy>()
            .sqrt()
    }
    pub fn chisquare(x: &Histogram, y: &Histogram) -> Energy {
        Abstraction::range()
            .map(|abstraction| (x.density(&abstraction), y.density(&abstraction)))
            .map(|(x, y)| (x - y).powi(2) / (x + y))
            .sum::<Energy>()
    }
    pub fn divergent(x: &Histogram, y: &Histogram) -> Energy {
        Abstraction::range()
            .map(|abstraction| (x.density(&abstraction), y.density(&abstraction)))
            .map(|(x, y)| (x - y).abs())
            .sum::<Energy>()
    }
}
