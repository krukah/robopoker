use mccfr::*;

/// Recovers the inner info set a subgame wrapper tagged.
pub trait Unwrap<I>
where
    I: CfrInfo,
{
    fn inner(&self) -> I;
}
