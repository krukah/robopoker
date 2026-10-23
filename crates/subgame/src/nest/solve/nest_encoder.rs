use crate::*;
use mccfr::*;

/// Wraps `&N`, lifting its info output into [`NestInfo`].
pub struct NestEncoder<'blueprint, N>
where
    N: Encoding,
    GameOf<N>: Augmentable,
{
    inner: &'blueprint N,
}

impl<'blueprint, N> NestEncoder<'blueprint, N>
where
    N: Encoding,
    GameOf<N>: Augmentable,
{
    pub fn new(inner: &'blueprint N) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &'blueprint N {
        self.inner
    }
}
