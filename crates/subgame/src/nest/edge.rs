//! Edge type for off-tree-augmented (nested) subgames.
use super::*;
use mccfr::*;
use monge::Support;
use pokerkit::*;

/// Extends a game's canonical edges with a single off-tree action — the
/// off-abstraction payload spliced in at the nesting entry node. Only ever
/// appears there (see [`super::NestPublic::Entry`]); canonical pipelines never
/// see it, so the base game's `Edge`/`Path` stay untouched.
///
/// Generic over the base edge `E` and the off-tree payload `Off` (chips, for
/// poker). Both are plain type parameters, so `derive` handles all the marker
/// traits — unlike [`super::NestGame`], which has an associated-type field.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum NestEdge<E, Off> {
    /// A canonical (on-grid) action.
    Game(E),
    /// An off-tree action carrying its literal payload.
    Off(Off),
}

impl<E, Off> NestEdge<E, Off>
where
    E: Copy,
    Off: Copy,
{
    /// The off-tree payload, if this is the off-tree edge.
    pub fn offtree(&self) -> Option<Off> {
        match self {
            Self::Game(_) => None,
            Self::Off(o) => Some(*o),
        }
    }
    /// The wrapped canonical edge, if this is a canonical edge.
    pub fn game(self) -> Option<E> {
        match self {
            Self::Game(e) => Some(e),
            Self::Off(_) => None,
        }
    }
}

impl<E, Off> Support for NestEdge<E, Off>
where
    E: CfrEdge,
    Off: OffPayload,
{
}
impl<E, Off> CfrEdge for NestEdge<E, Off>
where
    E: CfrEdge,
    Off: OffPayload,
{
    fn default_policy(&self) -> Probability {
        match self {
            Self::Game(e) => e.default_policy(),
            // No blueprint analog; 0.0 = uniform prior (the CfrEdge default).
            Self::Off(_) => 0.0,
        }
    }

    fn default_regret(&self) -> Utility {
        match self {
            Self::Game(e) => e.default_regret(),
            Self::Off(_) => 0.0,
        }
    }
}

impl<E, Off> std::fmt::Display for NestEdge<E, Off>
where
    E: std::fmt::Display,
    Off: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Game(e) => write!(f, "{e}"),
            Self::Off(o) => write!(f, "off({o})"),
        }
    }
}
