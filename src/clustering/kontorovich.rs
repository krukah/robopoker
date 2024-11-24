use super::histogram::Histogram;
use super::metric::Metric;

pub struct Kontorovich<'m, 'h> {
    lhs: &'h Histogram,
    rhs: &'h Histogram,
    metric: &'m Metric,
}
impl<'m, 'h> From<(&'m Metric, &'h Histogram, &'h Histogram)> for Kontorovich<'m, 'h> {
    fn from((metric, lhs, rhs): (&'m Metric, &'h Histogram, &'h Histogram)) -> Self {
        Self { metric, lhs, rhs }
    }
}
