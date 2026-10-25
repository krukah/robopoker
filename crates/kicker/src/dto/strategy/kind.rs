use pokerkit::Config;
use serde::{Deserialize, Serialize};

/// Server-side projection of [`pokerkit::Config`]'s `(depth, world)` axes to the route that produced an
/// [`crate::ApiSolved`]. Dirac lives off-wire — it's a client-side post-process.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Blueprint,
    Depth,
    World,
    Full,
}
impl Kind {
    /// Wire-format label, also used as the route segment and the FE
    /// provenance strip. Single source of truth so client and server
    /// agree.
    pub fn label(self) -> &'static str {
        match self {
            Self::Blueprint => "blueprint",
            Self::Depth => "depth",
            Self::World => "world",
            Self::Full => "full",
        }
    }
    /// Project a cube [`pokerkit::Config`] onto its route corner. `dirac` is
    /// post-processing on top of any of these — it doesn't pick a
    /// kind, it transforms the resulting distribution client-side.
    pub fn from_config(config: Config) -> Self {
        match (config.depth(), config.world()) {
            (false, false) => Self::Blueprint,
            (true, false) => Self::Depth,
            (false, true) => Self::World,
            (true, true) => Self::Full,
        }
    }
}
