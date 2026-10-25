//! Names the canonical solver configurations. A typed scaffold only — no
//! runtime dispatch: `Flagship` still drives
//! monomorphization, and only `Pluribus` is wired to it today.

use super::*;

/// Flagship NLHE solver, configured to match Pluribus (Brown & Sandholm,
/// Science 2019). [`mccfr::LinearRegret`] is Linear CFR — DCFR(1, 1, 1) — the
/// variant Pluribus actually used, and is a deliberate parity choice.
pub type Flagship = mccfr::Tabular<NlheEncoder, mccfr::LinearRegret, mccfr::LinearWeight, mccfr::PluribusSampling, 128>;

/// Named flagship solver configurations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlagshipKind {
    /// Pluribus-inspired: `Tabular<NlheEncoder, LinearRegret, LinearWeight, PluribusSampling, 128>`.
    #[default]
    Pluribus,
}

impl std::fmt::Display for FlagshipKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pluribus => write!(f, "pluribus"),
        }
    }
}
