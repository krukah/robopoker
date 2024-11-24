/// marker trait for any type that can
/// be interpreted as a support for a probability distribution.
///
/// currently only implemented by
/// - Abstraction::Random(_) , where Histogram is the implied Density and Metric is the implied Measure
/// - Abstraction::Equity(_) , where Histogram is the implied Density and i16    is the implied Measure
pub trait Support: Clone {}
