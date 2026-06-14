//! Validated, payload-carrying lattice on a typed scalar axis.

use std::marker::PhantomData;

use rand::Rng;

use crate::*;

/// A non-empty, strictly ascending sequence of finite f64 anchors,
/// each paired with a payload `P`, tagged with its [`Axis`].
///
/// Storage is a single `Box<[(f64, P)]>` — co-indexing of scalars and
/// payloads is **structural**, not an unwritten invariant.
///
/// `P` defaults to `()` for the no-payload case; `Lattice<A>` and
/// `Lattice<A, ()>` are the same type. Construct via [`FromIterator`]:
/// `pairs.into_iter().collect::<Lattice<A, P>>()` for the payload case
/// or `xs.into_iter().collect::<Lattice<A>>()` for the unit case.
pub struct Lattice<A, P = ()>
where
    A: Axis,
{
    pairs: Box<[(f64, P)]>,
    axis: PhantomData<A>,
}

impl<A, P> std::fmt::Debug for Lattice<A, P>
where
    A: Axis,
    P: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lattice").field("pairs", &self.pairs).finish()
    }
}

impl<A, P> Clone for Lattice<A, P>
where
    A: Axis,
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            pairs: self.pairs.clone(),
            axis: PhantomData,
        }
    }
}

impl<A, P> FromIterator<(f64, P)> for Lattice<A, P>
where
    A: Axis,
{
    /// Construct from `(scalar, payload)` pairs. Sorts by scalar
    /// internally. Panics on empty input, non-finite scalars, or
    /// duplicate keys — these are developer errors, not user input.
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (f64, P)>,
    {
        let mut pairs = iter.into_iter().collect::<Vec<_>>();
        assert!(!pairs.is_empty(), "Lattice must be non-empty");
        assert!(pairs.iter().all(|(s, _)| s.is_finite()), "Lattice scalars must be finite");
        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite scalar keys"));
        assert!(
            pairs.windows(2).all(|w| w[0].0 < w[1].0),
            "Lattice scalars must be strictly ascending (no duplicates)",
        );
        Self {
            pairs: pairs.into_boxed_slice(),
            axis: PhantomData,
        }
    }
}

impl<A> FromIterator<f64> for Lattice<A, ()>
where
    A: Axis,
{
    /// Construct a unit-payload lattice from bare scalars.
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = f64>,
    {
        iter.into_iter().map(|s| (s, ())).collect()
    }
}

impl<A, P> Lattice<A, P>
where
    A: Axis,
{
    /// Anchor count. Always `>= 1`.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// True iff the lattice has no anchors. Always `false` in practice
    /// (constructor guarantees `len >= 1`), exposed to satisfy clippy.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Iterator over scalars in ascending order.
    pub fn scalars(&self) -> impl ExactSizeIterator<Item = f64> + '_ {
        self.pairs.iter().map(|(s, _)| *s)
    }

    /// Payload at the given anchor.
    pub fn payload(&self, anchor: Anchor) -> &P {
        &self.pairs[anchor.idx()].1
    }

    /// Locate bracketing anchors `(lo, hi)` for `observed`. Returns
    /// `Bracket(a, a)` when clamped at an extreme, `Bracket(lo, hi)`
    /// with distinct indices when `observed` is strictly between two
    /// anchors.
    pub fn bracket(&self, observed: Scalar<A>) -> Bracket {
        let x = observed.value();
        let i = self.pairs.len() - 1;
        if x <= self.pairs[0].0 {
            Bracket::new(Anchor::new(0), Anchor::new(0))
        } else if x >= self.pairs[i].0 {
            Bracket::new(Anchor::new(i), Anchor::new(i))
        } else {
            Bracket::from(self.pairs.partition_point(|(s, _)| *s < x))
        }
    }
}

// --- Algorithms (dispatched by [`crate::Translation::resolve`]) ---
//
// All snap-family algorithms return [`Anchor`] directly. Brown-style
// injection variants (future PR B) will return `Option<Anchor>` and let
// the dispatcher lift to [`Translated::Free`] when off-grid.

impl<A, P> Lattice<A, P>
where
    A: Axis,
{
    pub fn snap(&self, observed: Scalar<A>) -> Anchor {
        let x = observed.value();
        let i = self
            .scalars()
            .enumerate()
            .min_by(|(_, a), (_, b)| (a - x).abs().partial_cmp(&(b - x).abs()).expect("finite anchors"))
            .map(|(i, _)| i)
            .expect("non-empty lattice");
        Anchor::new(i)
    }

    /// Pseudo-harmonic conditional probability of the lower anchor for
    /// observation `x` bracketed by `(lo, hi)`. Encodes the
    /// Ganzfried-Sandholm 2013 formula `p = (B-x)(1+A) / (B-A)(1+x)`.
    ///
    /// The `(1+x)` term assumes the axis is non-negative — see [`Axis`].
    /// The `lo != hi` precondition is asserted; callers must check
    /// `Bracket::is_clamped` first.
    pub fn pharmonic(&self, bracket: Bracket, observed: Scalar<A>) -> f64 {
        if bracket.is_clamped() {
            unreachable!("pharmonic requires distinct bracketing anchors")
        } else {
            let a = self.pairs[bracket.lo().idx()].0;
            let b = self.pairs[bracket.hi().idx()].0;
            let x = observed.value();
            ((b - x) * (1.0 + a)) / ((b - a) * (1.0 + x))
        }
    }

    pub fn harmonic<R>(&self, observed: Scalar<A>, rng: &mut R) -> Anchor
    where
        R: Rng + ?Sized,
    {
        let bracket = self.bracket(observed);
        if bracket.is_clamped() || rng.random::<f64>() < self.pharmonic(bracket, observed) {
            bracket.lo()
        } else {
            bracket.hi()
        }
    }

    pub fn phargmax(&self, observed: Scalar<A>) -> Anchor {
        let bracket = self.bracket(observed);
        if bracket.is_clamped() || self.pharmonic(bracket, observed) >= 0.5 {
            bracket.lo()
        } else {
            bracket.hi()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct T;
    impl Axis for T {}

    fn obs(x: f64) -> Scalar<T> {
        Scalar::new(x)
    }

    fn lat(xs: impl IntoIterator<Item = f64>) -> Lattice<T> {
        xs.into_iter().collect()
    }

    #[test]
    fn from_scalars_constructs_unit_payload() {
        let l = lat([0.5, 1.0, 2.0]);
        assert_eq!(l.scalars().collect::<Vec<_>>(), vec![0.5, 1.0, 2.0]);
        assert_eq!(l.len(), 3);
    }

    #[test]
    fn from_iter_sorts_unsorted_pairs() {
        let l = [(2.0, "two"), (0.5, "half"), (1.0, "one")]
            .into_iter()
            .collect::<Lattice<T, _>>();
        assert_eq!(l.scalars().collect::<Vec<_>>(), vec![0.5, 1.0, 2.0]);
        assert_eq!(*l.payload(Anchor::new(0)), "half");
        assert_eq!(*l.payload(Anchor::new(2)), "two");
    }

    #[test]
    #[should_panic(expected = "non-empty")]
    fn rejects_empty() {
        let _: Lattice<T> = std::iter::empty::<f64>().collect();
    }

    #[test]
    #[should_panic(expected = "finite")]
    fn rejects_non_finite() {
        let _: Lattice<T, _> = [(0.0, ()), (f64::NAN, ()), (1.0, ())].into_iter().collect();
    }

    #[test]
    #[should_panic(expected = "ascending")]
    fn rejects_duplicate_keys() {
        let _: Lattice<T, _> = [(1.0, 'a'), (1.0, 'b')].into_iter().collect();
    }

    #[test]
    fn bracket_clamps_below_extreme() {
        let l = lat([0.5, 1.0, 2.0]);
        let b = l.bracket(obs(0.1));
        assert!(b.is_clamped());
        assert_eq!(b.lo(), Anchor::new(0));
        let b = l.bracket(obs(0.5));
        assert!(b.is_clamped());
        assert_eq!(b.lo(), Anchor::new(0));
    }

    #[test]
    fn bracket_clamps_above_extreme() {
        let l = lat([0.5, 1.0, 2.0]);
        let b = l.bracket(obs(2.0));
        assert!(b.is_clamped());
        assert_eq!(b.lo(), Anchor::new(2));
        let b = l.bracket(obs(3.0));
        assert!(b.is_clamped());
        assert_eq!(b.lo(), Anchor::new(2));
    }

    #[test]
    fn bracket_inside_returns_distinct_anchors() {
        let l = lat([0.5, 1.0, 2.0]);
        let b = l.bracket(obs(0.75));
        assert!(!b.is_clamped());
        assert_eq!(b.lo(), Anchor::new(0));
        assert_eq!(b.hi(), Anchor::new(1));
    }

    #[test]
    fn pharmonic_formula_exact() {
        let l = lat([0.5, 1.0]);
        let bracket = l.bracket(obs(0.75));
        let expected = (1.0 - 0.75) * (1.0 + 0.5) / ((1.0 - 0.5) * (1.0 + 0.75));
        let p = l.pharmonic(bracket, obs(0.75));
        assert!((p - expected).abs() < 1e-12);
    }
}
