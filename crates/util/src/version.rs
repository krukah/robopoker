//! Abstraction version controlling clustering parameters and table names.
//!
//! Versions form one axis of the (Version × Regime) training configuration:
//! they suffix abstraction-layer tables (isomorphism, abstraction, street,
//! transitions). Each version represents a distinct run of hierarchical
//! k-means clustering, potentially with different K values, distance
//! metrics, or street hierarchies.

/// Abstraction version controlling clustering parameters and table names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Version {
    /// The initial abstraction family. Bare table names (no suffix) for
    /// backwards compatibility with prod.
    #[default]
    V0,
    /// K=256 clustering with debiased Sinkhorn metric. Tables suffixed `_v1`.
    V1,
}

static VERSION: std::sync::OnceLock<Version> = std::sync::OnceLock::<Version>::new();

/// Returns the active version. Defaults to V0 if `init_version` was never called.
pub fn version() -> Version {
    *VERSION.get_or_init(|| Version::V0)
}

/// Sets the active version. Must be called before any table access.
/// Panics if called twice with different values.
pub fn init_version(v: Version) {
    if let Err(existing) = VERSION.set(v) {
        assert_eq!(
            existing, v,
            "version already set to {:?}, cannot change to {:?}",
            existing, v
        );
    }
}

impl Version {
    /// Database table suffix for this version.
    /// V0 uses no suffix for backwards compatibility with existing tables.
    pub fn suffix(self) -> &'static str {
        match self {
            Self::V0 => "",
            Self::V1 => "_v1",
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V0 => write!(f, "v0"),
            Self::V1 => write!(f, "v1"),
        }
    }
}
