//! The bot zoo — runtime → comptime binding for the bot-config hypercube.
//!
//! Three binary axes (depth-limit, world-choice, dirac) give 8 concrete
//! [`Brain`](super::Brain) compositions. All three are wrappers — depth
//! and world add subgame solver layers, [`Dirac`](super::Dirac) sharpens
//! to a Dirac delta. The [`Agent`](super::Agent) wraps the chosen
//! [`Brain`] and implements [`Player`].
//!
//! Adding the harmonic axis = double the cells (16) and add another
//! match dimension. The grid below aligns each axis token at its
//! position in the bottom row's `Dirac<World<Depth<Blueprint>>>` —
//! reading column-by-column tells you which axis is active.
use pokerkit::Config;
use vitals::KeyValue;

use super::Agent;
use super::Blueprint;
use super::Depth;
use super::Dirac;
use super::Nest;
use super::World;
use crate::Player;
use nlhe::Flagship;
use pokerkit::Translation;

/// A bot's display identity + cube coordinate. Threaded through the
/// [`Mount`](super::Mount) cascade so every emission site (metrics,
/// tracing) can lift the same triple of axis labels (`depth`, `world`,
/// `dirac`) onto Prometheus series alongside the composite `variant`
/// label. Lets Grafana group/filter by axis and compute corner-pair
/// diffs (e.g. marginal value of depth-limiting averaged over the
/// `world × dirac` plane) directly in PromQL.
#[derive(Copy, Clone, Debug)]
pub struct Tag {
    pub label: &'static str,
    pub config: Config,
}

impl Tag {
    /// The four OTLP labels every cube-cell metric carries: composite
    /// `variant` for stable color/legend rules + DB joins, plus three
    /// axis labels for cube slicing. Each axis label name + value
    /// matches the [`Config`] field 1:1 (`depth`/`world`/`dirac`,
    /// each `on`/`off`) so dashboard PromQL never has to translate.
    /// Append metric-specific keys (e.g. `street`) at the call site.
    pub fn keys(&self) -> [KeyValue; 4] {
        [
            KeyValue::new("variant", self.label),
            KeyValue::new("depth", if self.config.depth { "on" } else { "off" }),
            KeyValue::new("world", if self.config.world { "on" } else { "off" }),
            KeyValue::new("dirac", if self.config.dirac { "on" } else { "off" }),
        ]
    }
}

/// Look up a bot in the zoo by its [`Tag`]. Each arm monomorphizes its
/// own hot path; the match itself runs once at startup.
///
/// The `nest` (Modicum off-tree re-solve) layer is *not* a [`Config`] axis —
/// it's a runtime consequence of the active [`Translation`]. Off-tree raises
/// only exist under [`Translation::Exact`], and nesting is a world-subgame
/// re-solve, so a world bot running under `Exact` gets wrapped in
/// [`Nest`](super::Nest) (inside `Dirac`, outside `World`). This keeps `nest`
/// off the user-facing `Variant`/`Config` wire format (and out of the WASM
/// client) — the translation flag alone turns it on.
#[rustfmt::skip]
pub fn zoo(tag: Tag, model: &'static Flagship) -> Box<dyn Player> {
    let nest = matches!(pokerkit::translation(), Translation::Exact) && tag.config.world;
    match (tag.config.depth, tag.config.world, tag.config.dirac, nest) {
        // nest layer active: Exact translation + world bot → wrap the world stack.
        (false, true,  false, true ) => Agent::<      Nest<World<      Blueprint >>>::boxed(tag, model),
        (false, true,  true,  true ) => Agent::<Dirac<Nest<World<      Blueprint >>>>::boxed(tag, model),
        (true,  true,  false, true ) => Agent::<      Nest<World<Depth<Blueprint>>>>::boxed(tag, model),
        (true,  true,  true,  true ) => Agent::<Dirac<Nest<World<Depth<Blueprint>>>>>::boxed(tag, model),
        // no nest (not Exact, or not a world bot) → the canonical 8 cells.
        (false, false, false, _    ) => Agent::<                  Blueprint   >::boxed(tag, model),
        (false, false, true,  _    ) => Agent::<Dirac<            Blueprint  >>::boxed(tag, model),
        (true,  false, false, _    ) => Agent::<            Depth<Blueprint>  >::boxed(tag, model),
        (true,  false, true,  _    ) => Agent::<Dirac<      Depth<Blueprint> >>::boxed(tag, model),
        (false, true,  false, false) => Agent::<      World<      Blueprint > >::boxed(tag, model),
        (false, true,  true,  false) => Agent::<Dirac<World<      Blueprint >>>::boxed(tag, model),
        (true,  true,  false, false) => Agent::<      World<Depth<Blueprint>> >::boxed(tag, model),
        (true,  true,  true,  false) => Agent::<Dirac<World<Depth<Blueprint>>>>::boxed(tag, model),
    }
}
