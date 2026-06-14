//! Axis-tagged scalar observation.

use std::marker::PhantomData;

use crate::*;

/// A finite scalar observation tagged with its [`Axis`].
///
/// Named `Scalar` rather than `Observation` to avoid colliding with the
/// poker-domain `kicker::Observation`.
pub struct Scalar<A>
where
    A: Axis,
{
    value: f64,
    axis: PhantomData<A>,
}

impl<A> std::fmt::Debug for Scalar<A>
where
    A: Axis,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scalar").field("value", &self.value).finish()
    }
}

impl<A> Copy for Scalar<A> where A: Axis {}

impl<A> Clone for Scalar<A>
where
    A: Axis,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Scalar<A>
where
    A: Axis,
{
    /// Construct a scalar. Panics if not finite — callers are expected
    /// to validate at the system boundary, not pass NaN/inf in.
    pub fn new(value: f64) -> Self {
        assert!(value.is_finite(), "Scalar must be finite, got {value}");
        Self {
            value,
            axis: PhantomData,
        }
    }

    /// Raw value.
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct T;
    impl Axis for T {}

    #[test]
    #[should_panic(expected = "Scalar must be finite")]
    fn rejects_nan() {
        let _ = Scalar::<T>::new(f64::NAN);
    }

    #[test]
    #[should_panic(expected = "Scalar must be finite")]
    fn rejects_infinity() {
        let _ = Scalar::<T>::new(f64::INFINITY);
    }
}
