//! Abstraction version controlling clustering parameters and table names.
//!
//! Versions form one axis of the (Version × Regime) training configuration:
//! they suffix abstraction-layer tables (isomorphism, abstraction, street,
//! transitions). Each version represents a distinct run of hierarchical
//! k-means clustering, potentially with different K values, distance
//! metrics, or street hierarchies.
//!
//! V3 is the live writable version. V0/V1/V2 remain in the enum so tooling
//! can address their DB tables, but the live training/serving codepath
//! (post-SPR-cutover) only writes V3 — V0/V1/V2 tables are cold storage.

/// Abstraction version controlling clustering parameters and table names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Version {
    /// The initial abstraction family. Bare table names (no suffix) for
    /// backwards compatibility with prod. Read-only: blueprint schema
    /// predates the SPR axis and the v3 cutover.
    V0,
    /// K=256 clustering with debiased Sinkhorn metric. Tables suffixed
    /// `_v1`. Clustering tables here are still the source of truth for
    /// V3 (see [`Self::clustering_suffix`]).
    V1,
    /// SPR-keyed action grid. Blueprint tables suffixed `_v2`. Read-only
    /// under v3 code — blueprint schema includes the `geometry` column
    /// which v3 no longer writes.
    V2,
    /// Pluribus-faithful grid with no SPR axis on the InfoSet key.
    /// Blueprint tables suffixed `_v3`; clustering tables continue to be
    /// read from the `_v1` suffix via [`Self::clustering_suffix`].
    #[default]
    V3,
}

static VERSION: std::sync::OnceLock<Version> = std::sync::OnceLock::<Version>::new();

/// Returns the active version. Defaults to V3.
pub fn version() -> Version {
    *VERSION.get_or_init(|| Version::V3)
}

/// Sets the active version. Must be called before any table access.
/// Panics if called twice with different values.
pub fn init_version(v: Version) {
    if let Err(existing) = VERSION.set(v) {
        assert_eq!(existing, v, "version already set to {existing:?}, cannot change to {v:?}");
    }
}

impl Version {
    /// Database table suffix for this version.
    /// V0 uses no suffix for backwards compatibility with existing tables.
    pub fn suffix(self) -> &'static str {
        match self {
            Self::V0 => "",
            Self::V1 => "_v1",
            Self::V2 => "_v2",
            Self::V3 => "_v3",
        }
    }

    /// Suffix of the clustering tables this version reads from.
    ///
    /// Clustering tables (`abstraction`, `isomorphism`, `metric`, `street`,
    /// `transitions`) are expensive to recompute and depend only on
    /// K-means / Sinkhorn parameters — not on the bet-sizing grid. When a
    /// new `Version` only changes the grid (V2 & V3 both reuse V1's
    /// clustering), it reads the existing `_v1` clustering tables.
    pub fn clustering_suffix(self) -> &'static str {
        match self {
            Self::V0 => "",
            Self::V1 | Self::V2 | Self::V3 => "_v1",
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V0 => write!(f, "v0"),
            Self::V1 => write!(f, "v1"),
            Self::V2 => write!(f, "v2"),
            Self::V3 => write!(f, "v3"),
        }
    }
}
