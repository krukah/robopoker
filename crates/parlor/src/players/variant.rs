//! Gameroom-side extensions to [`pokerkit::Variant`].
//!
//! The pure data shape of `Variant` lives in `pokerkit` so the WASM
//! client can use it. This module adds the methods that need
//! gameroom-only types: constructing a [`Player`] via [`zoo`], building
//! a [`Member`] / [`User`] for the room sit-down, and emitting OTLP
//! [`KeyValue`]s for telemetry.
use crate::Player;

use super::Fish;
use super::Tag;
use super::zoo;
use bouncer::Member;
use bouncer::User;
use pokerkit::Config;
use pokerkit::ID;
use pokerkit::Variant;
use pokerkit::slumbot_opponent_username;
use pokerkit::slumbot_opponent_uuid;
use vitals::KeyValue;

/// Gameroom-only methods on [`Variant`]. Implemented as an extension
/// trait because the type lives in `pokerkit` (which the WASM client
/// reads) and these methods need types from `bouncer`, `holdem`,
/// `vitals` that the client must not pull in.
pub trait VariantExt {
    fn tag(self) -> Option<Tag>;
    fn email(self) -> String;
    fn id(self) -> ID<Member>;
    fn member(self) -> Member;
    fn user(self) -> User;
    fn keys(self) -> [KeyValue; 4];
    fn into_player(self, flagship: Option<&'static holdem::Flagship>) -> Box<dyn Player>;
}

impl VariantExt for Variant {
    /// Telemetry tag carrying both the canonical label and the cube
    /// triple. `None` for fish.
    fn tag(self) -> Option<Tag> {
        self.config().map(|config: Config| Tag {
            label: self.label(),
            config,
        })
    }
    fn email(self) -> String {
        format!("{}@robopoker.io", self.label())
    }
    fn id(self) -> ID<Member> {
        ID::from(self.uuid())
    }
    fn member(self) -> Member {
        Member::new(self.id(), self.username(), self.email())
    }
    fn user(self) -> User {
        User::Auth(self.member())
    }
    /// OTLP labels for slumbot benchmark / room metrics. Bot variants
    /// emit the cube coordinates; fish emits axis values of `"n/a"` so
    /// every series carries the same label cardinality.
    fn keys(self) -> [KeyValue; 4] {
        match self.tag() {
            Some(tag) => tag.keys(),
            None => [
                KeyValue::new("variant", "fish"),
                KeyValue::new("depth", "n/a"),
                KeyValue::new("world", "n/a"),
                KeyValue::new("dirac", "n/a"),
            ],
        }
    }
    /// Build the concrete `Player` for this variant. Consumes `self` —
    /// each Variant materializes exactly one player.
    fn into_player(self, flagship: Option<&'static holdem::Flagship>) -> Box<dyn Player> {
        match self.tag() {
            None => Box::new(Fish),
            Some(tag) => zoo(tag, flagship.expect("bot variant requires flagship")),
        }
    }
}

/// Pseudo-member representing slumbot.com when the slumbot crate
/// records hands against the external API. Not in the zoo.
pub fn slumbot_opponent() -> Member {
    Member::new(
        ID::from(slumbot_opponent_uuid()),
        slumbot_opponent_username().to_string(),
        format!("{}@robopoker.io", slumbot_opponent_username().trim_start_matches("bot:")),
    )
}

/// Convenience for callsites that only have an `ID<Member>`.
pub fn slumbot_opponent_id() -> ID<Member> {
    ID::from(slumbot_opponent_uuid())
}
