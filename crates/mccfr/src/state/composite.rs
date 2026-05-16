//! Product type for composing a [`CfrPublic`] and a [`CfrSecret`] into a [`CfrInfo`].
//!
//! Most games want their info set to be a straightforward `(public, secret)` pair.
//! `Composite<X, Y>` exposes that product with a blanket [`CfrInfo`] impl so game
//! crates don't have to hand-roll the `public() -> X` / `secret() -> Y` delegation.
//!
//! Games that need custom info identity (e.g. holdem uses this for NLHE info sets
//! via a thin newtype) still compose the same way.

use crate::*;

/// Product of a public and a secret component, usable as a [`CfrInfo`].
///
/// Semantics: two composites are equal iff both components are equal; hashing
/// and ordering fall out component-wise. Display prints `{secret}|{public}`
/// when both components are `Display`, matching the convention used by the
/// concrete poker games.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Composite<X, Y> {
    public: X,
    secret: Y,
}

impl<X, Y> Composite<X, Y> {
    pub fn new(public: X, secret: Y) -> Self {
        Self { public, secret }
    }
}

impl<X, Y> CfrInfo for Composite<X, Y>
where
    X: CfrPublic,
    Y: CfrSecret,
{
    type E = X::E;
    type T = X::T;
    type X = X;
    type Y = Y;

    fn public(&self) -> Self::X {
        self.public
    }

    fn secret(&self) -> Self::Y {
        self.secret
    }
}

impl<X, Y> std::fmt::Display for Composite<X, Y>
where
    X: std::fmt::Display,
    Y: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.secret, self.public)
    }
}
