//! Abstraction version controlling clustering parameters and table names.
//!
//! Versions form one axis of the (Version × Regime) training configuration:
//! they suffix abstraction-layer tables (isomorphism, abstraction, street,
//! transitions). Each version represents a distinct run of hierarchical
//! k-means clustering, potentially with different K values, distance
//! metrics, or street hierarchies.
//!
//! V1 is the sole version. It carries the pluribus-faithful design and reads
//! the (bug-free, deterministic) `_v1` clustering tables.

/// Abstraction version controlling clustering parameters and table names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Version {
    /// K=256 clustering with debiased Sinkhorn metric. Tables suffixed
    /// `_v1`. The only live version; carries the pluribus-faithful grid
    /// with no SPR axis on the InfoSet key.
    #[default]
    V1,
}

static VERSION: std::sync::OnceLock<Version> = std::sync::OnceLock::<Version>::new();

/// Returns the active version. Defaults to V1.
pub fn version() -> Version {
    *VERSION.get_or_init(|| Version::V1)
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
    pub fn suffix(self) -> &'static str {
        match self {
            Self::V1 => "_v1",
        }
    }

    /// Suffix of the clustering tables this version reads from.
    ///
    /// Clustering tables (`abstraction`, `isomorphism`, `metric`, `street`,
    /// `transitions`) are expensive to recompute and depend only on
    /// K-means / Sinkhorn parameters — not on the bet-sizing grid. They are
    /// unaffected by the MCCFR sampling-weight bug and are reused as-is.
    pub fn clustering_suffix(self) -> &'static str {
        match self {
            Self::V1 => "_v1",
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "v1"),
        }
    }
}
