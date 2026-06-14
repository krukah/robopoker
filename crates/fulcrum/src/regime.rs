//! Action abstraction regime controlling bet sizing grid and database table names.
//!
//! Regimes form the other axis of the (Version × Regime) training configuration:
//! they suffix policy-layer tables (blueprint, epoch, snapshot). Every regime
//! gets its own suffixed tables — no regime is "default."

/// Action abstraction regime controlling bet sizing grid and database table names.
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

/// Returns the active regime. Defaults to Pluribus if `init_regime` was never called.
/// Binaries should still require explicit `--regime` via clap.
pub fn regime() -> Regime {
    *REGIME.get_or_init(|| Regime::Pluribus)
}

/// Sets the active regime. Must be called before any sizing or table access.
/// Panics if called twice with different values.
pub fn init_regime(r: Regime) {
    if let Err(existing) = REGIME.set(r) {
        assert_eq!(existing, r, "regime already set to {existing:?}, cannot change to {r:?}");
    }
}

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

/// Compact textual fingerprint of every constant that defines the game tree
/// shape for this regime. Two trainers producing the same fingerprint can
/// safely share the same `blueprint_<regime>_<version>` table; two that
/// don't would silently corrupt each other's strategy data — InfoIds keyed
/// by `Edge::Raise(Size)` carry the same byte representation but mean
/// different actions when the underlying `Size` lattice shifts.
///
/// **Add a constant here when you add one whose change should invalidate
/// existing blueprints.** Anything you forget to add is silent drift the
/// runtime check at trainer startup will not catch.
pub fn config_string(r: Regime) -> String {
    use crate::*;
    let common = format!(
        "STACK={STACK};B_BLIND={B_BLIND};S_BLIND={S_BLIND};MAX_RAISE_REPEATS={MAX_RAISE_REPEATS};OPENS={OPENS:?}",
    );
    match r {
        Regime::Pluribus => format!("{common};PLURIBUS_INDICES={PLURIBUS_INDICES:?};RAISES={RAISES:?}"),
        Regime::Slumbot => format!("{common};SLUMBOT_INDICES={SLUMBOT_INDICES:?};RAISES={RAISES:?}"),
    }
}
