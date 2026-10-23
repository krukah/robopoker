//! The bot zoo — runtime → comptime binding for the bot-config hypercube.
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

/// A bot's display identity + cube coordinate.
///
/// Threaded through the [`Mount`](super::Mount) cascade so every emission site lifts the same
/// axis labels (`depth`/`world`/`dirac`) alongside the composite `variant`.
#[derive(Copy, Clone, Debug)]
pub struct Tag {
    label: &'static str,
    config: Config,
}

impl Tag {
    pub fn new(label: &'static str, config: Config) -> Self {
        Self { label, config }
    }

    /// The four OTLP labels every cube-cell metric carries: composite `variant` (stable legend
    /// rules + DB joins) plus the three axis labels. Each axis label name and value matches its
    /// [`Config`] field 1:1 (`on`/`off`) so dashboard PromQL never translates.
    pub fn keys(&self) -> [KeyValue; 4] {
        [
            KeyValue::new("variant", *self.label()),
            KeyValue::new("depth", if self.config().depth() { "on" } else { "off" }),
            KeyValue::new("world", if self.config().world() { "on" } else { "off" }),
            KeyValue::new("dirac", if self.config().dirac() { "on" } else { "off" }),
        ]
    }
}

impl Tag {
    pub fn label(&self) -> &&'static str {
        &self.label
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Tag {
    /// Look up a bot in the zoo by its [`Tag`]. Each arm monomorphizes its own hot path; the match
    /// itself runs once at startup.
    #[rustfmt::skip]
    pub fn zoo(tag: Tag, model: &'static Flagship) -> Box<dyn Player> {
        let nest = matches!(pokerkit::Translation::current(), Translation::Exact) && tag.config().world();
        // The first four cells wrap the world stack in the nest layer; the rest
        // are the canonical eight.
        match (tag.config().depth(), tag.config().world(), tag.config().dirac(), nest) {
            (false, true,  false, true ) => Agent::<      Nest<World<      Blueprint >>>::boxed(tag, model),
            (false, true,  true,  true ) => Agent::<Dirac<Nest<World<      Blueprint >>>>::boxed(tag, model),
            (true,  true,  false, true ) => Agent::<      Nest<World<Depth<Blueprint>>>>::boxed(tag, model),
            (true,  true,  true,  true ) => Agent::<Dirac<Nest<World<Depth<Blueprint>>>>>::boxed(tag, model),
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
}
