//! Action abstraction regime controlling bet sizing grid and table names.
//!
//! One axis of the (Version × Regime) training configuration: regimes suffix
//! policy-layer tables (blueprint, epoch, snapshot). Every regime gets its own
//! suffixed tables — no regime is "default."

/// Action abstraction regime controlling bet sizing grid and table names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Regime {
    /// Pluribus-inspired bet sizing grid (Brown & Sandholm 2019).
    #[default]
    Pluribus,
    /// Slumbot-matching bet sizing grid (minimizes translation loss).
    Slumbot,
}

static REGIME: std::sync::OnceLock<Regime> = std::sync::OnceLock::<Regime>::new();

impl Regime {
    /// Database table suffix for this regime.
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Pluribus => "_pluribus",
            Self::Slumbot => "_slumbot",
        }
    }
}

impl std::fmt::Display for Regime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pluribus => write!(f, "pluribus"),
            Self::Slumbot => write!(f, "slumbot"),
        }
    }
}

impl Regime {
    /// Returns the active regime. Defaults to Pluribus if `init_regime` was never called.
    /// Binaries should still require explicit `--regime` via clap.
    pub fn current() -> Regime {
        *REGIME.get_or_init(|| Regime::Pluribus)
    }

    /// Sets the active regime. Must be called before any sizing or table access.
    /// Panics if called twice with different values.
    pub fn init_regime(r: Regime) {
        if let Err(existing) = REGIME.set(r) {
            assert_eq!(existing, r, "regime already set to {existing:?}, cannot change to {r:?}");
        }
    }
}
