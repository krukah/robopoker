use super::support::Support;

/// generalization of *element-wise* distance metric between
/// two Density spaces over arbitrary Support.
///
/// for Equity, this is trivially the absolute value of the difference.
/// for Metric, we precompute distances based on the previous layer's clustering.
///
/// in both these cases, X and Y are of the same type.
/// note however that generally, image space X and range space Y need not
/// have the same support. what is important is that we can define a
/// distance between any x ∈ X and any y ∈ Y.
pub trait Measure {
    type X: Support;
    type Y: Support;
    fn distance(&self, x: &Self::X, y: &Self::Y) -> f32;
}
