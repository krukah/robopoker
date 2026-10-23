#![allow(dead_code)]

// TRAITS
/// Random instance generation for testing and Monte Carlo sampling.
pub trait Arbitrary {
    fn random() -> Self;
}
