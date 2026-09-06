//! Names the canonical solver configurations. A typed scaffold only — no
//! runtime dispatch: `pub type Flagship = Nlhe<...>` still drives
//! monomorphization, and only `Pluribus` is wired to it today.

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
