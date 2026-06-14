//! Secret type alias extracted from an encoder's info type.
use regret::CfrEncoder;
use regret::CfrInfo;

/// The secret type for an encoder, extracted through its info set.
pub type Secret<N> = <<N as CfrEncoder>::I as CfrInfo>::Y;
