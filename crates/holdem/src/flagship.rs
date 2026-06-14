//! Flagship solver variant scaffold.
//!
//! Names the canonical solver configurations. Today only `Pluribus` is wired
//! to the `Flagship` type alias; future variants (Discounted, CFR+) will be
//! added here and dispatched through `FlagshipConfig`.
//!
//! No runtime dispatch yet — this is a typed enum scaffold for the top-level
//! `Config` to compose. The existing `pub type Flagship = Nlhe<...>` alias
//! continues to drive monomorphization.

/// Named flagship solver configurations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlagshipKind {
    /// Pluribus-inspired: `Nlhe<LinearRegret, LinearWeight, PluribusSampling>`.
    #[default]
    Pluribus,
    // Future: Discounted, CFRPlus, ...
}

impl std::fmt::Display for FlagshipKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pluribus => write!(f, "pluribus"),
        }
    }
}
