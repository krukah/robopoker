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
        assert_eq!(
            existing, r,
            "regime already set to {:?}, cannot change to {:?}",
            existing, r
        );
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
    // SPR bucket boundaries — keep this constant in sync with
    // `rbp_holdem::Geometry::BOUNDARIES`. The geometry feature is part of
    // the infoset key, so changing the boundaries silently re-buckets
    // existing rows; the fingerprint catches that at startup.
    const GEOMETRY_BOUNDARIES: [f32; 4] = [1.5, 4.0, 10.0, 30.0];
    let common = format!(
        "STACK={};B_BLIND={};S_BLIND={};MAX_RAISE_REPEATS={};OPENS={:?};GEOMETRY={:?}",
        STACK, B_BLIND, S_BLIND, MAX_RAISE_REPEATS, OPENS, GEOMETRY_BOUNDARIES,
    );
    match r {
        Regime::Pluribus => format!(
            "{common};\
             SIZE_PREF_1={:?};SIZE_PREF_N={:?};\
             SIZE_FLOP_0={:?};SIZE_FLOP_1={:?};SIZE_FLOP_N={:?};\
             SIZE_TURN_0={:?};SIZE_TURN_1={:?};SIZE_TURN_N={:?};\
             SIZE_RIVE_0={:?};SIZE_RIVE_1={:?};SIZE_RIVE_N={:?}",
            SIZE_PREF_1, SIZE_PREF_N,
            SIZE_FLOP_0, SIZE_FLOP_1, SIZE_FLOP_N,
            SIZE_TURN_0, SIZE_TURN_1, SIZE_TURN_N,
            SIZE_RIVE_0, SIZE_RIVE_1, SIZE_RIVE_N,
        ),
        Regime::Slumbot => format!("{common};SLUMBOT_SIZES={:?}", SLUMBOT_SIZES),
    }
}
