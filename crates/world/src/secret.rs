//! Secret type alias extracted from an encoder's info type.
use rbp_mccfr::CfrEncoder;
use rbp_mccfr::CfrInfo;

/// The secret type for an encoder, extracted through its info set.
pub type Secret<N> = <<N as CfrEncoder>::I as CfrInfo>::Y;
