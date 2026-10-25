//! Secret type alias extracted from an encoder's info type.
use mccfr::*;

/// The secret type for any spec, extracted through its info set.
pub type Secret<N> = <InfoOf<N> as CfrInfo>::Y;
