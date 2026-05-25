//! The cube grain — single shape used everywhere the user picks a bot.
//!
//! Hosting endpoint, slumbot CLI, and the gameplay UI all
//! deserialize / parse into the same [`Variant`]. No name layer
//! between the cube and the rest of the system.
//!
//! Pure data lives here in [`rbp_core`] so the WASM client can use it
//! without dragging in the database/holdem feature graph. Gameroom-side
//! extensions (`into_player`, `member`, telemetry keys) live in
//! `rbp_gameroom::players::variant` and read this type.
//!
//! ```text
//! Variant := Fish | Bot { depth: bool, world: bool, dirac: bool }
//! ```
//!
//! # Wire format (serde)
//!
//! ```json
//! { "kind": "fish" }
//! { "kind": "bot", "depth": false, "world": true, "dirac": false }
//! ```
//!
//! # CLI grammar
//!
//! - `fish` — random opponent (no model, not in the cube)
//! - `base` — the empty flag-set cube cell (raw blueprint sample)
//! - `+`-joined flags from `{depth, world, dirac}` in canonical order:
//!   `depth`, `world`, `dirac`, `depth+world`, `depth+dirac`,
//!   `world+dirac`, `depth+world+dirac`
//!
//! # Identity
//!
//! Every variant has a stable username `bot:<label>` and a UUID v5
//! derived from that username. The cube IS the identity.

/// Namespace UUID v5 for deterministic bot identity. Combined with the
/// `bot:<label>` username, gives every cube cell + fish a stable UUID
/// independent of the build (server, slumbot, client).
pub const VARIANT_NAMESPACE: uuid::Uuid = uuid::Uuid::from_bytes([
    0x72, 0x6f, 0x62, 0x6f, 0x70, 0x6f, 0x6b, 0x65, 0x72, 0x2e, 0x62, 0x6f, 0x74, 0x73, 0x2e, 0x76,
]);

/// The three boolean axes that pick out one cell of the zoo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub depth: bool,
    pub world: bool,
    pub dirac: bool,
}

/// Who's playing — picked by the user (UI), CLI (slumbot), or the
/// hosting backend. The cube IS the identity; the label and UUID are
/// derived from the axis triple.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[derive(Default)]
pub enum Variant {
    #[default]
    Fish,
    Bot {
        depth: bool,
        world: bool,
        dirac: bool,
    },
}

impl Variant {
    /// Canonical token for this variant. Stable: matches the CLI parse
    /// grammar and the DB username suffix (`bot:<label>`).
    #[rustfmt::skip]
    pub fn label(self) -> &'static str {
        match self {
            Self::Fish                                                 => "fish",
            Self::Bot { depth: false, world: false, dirac: false } => "base",
            Self::Bot { depth: true,  world: false, dirac: false } => "depth",
            Self::Bot { depth: false, world: true,  dirac: false } => "world",
            Self::Bot { depth: false, world: false, dirac: true  } => "dirac",
            Self::Bot { depth: true,  world: true,  dirac: false } => "depth+world",
            Self::Bot { depth: true,  world: false, dirac: true  } => "depth+dirac",
            Self::Bot { depth: false, world: true,  dirac: true  } => "world+dirac",
            Self::Bot { depth: true,  world: true,  dirac: true  } => "depth+world+dirac",
        }
    }
    /// `Some(Config)` for cube cells, `None` for fish.
    pub fn config(self) -> Option<Config> {
        match self {
            Self::Fish => None,
            Self::Bot { depth, world, dirac } => Some(Config { depth, world, dirac }),
        }
    }
    /// One-line description for UI tooltips and history filter rows.
    #[rustfmt::skip]
    pub fn description(self) -> &'static str {
        match self {
            Self::Fish                                                 => "Random actions",
            Self::Bot { depth: false, world: false, dirac: false } => "Sampled blueprint",
            Self::Bot { depth: true,  world: false, dirac: false } => "Depth-limited subgame",
            Self::Bot { depth: false, world: true,  dirac: false } => "World-partitioned subgame",
            Self::Bot { depth: false, world: false, dirac: true  } => "Argmax blueprint",
            Self::Bot { depth: true,  world: true,  dirac: false } => "Depth + world subgame",
            Self::Bot { depth: true,  world: false, dirac: true  } => "Argmax + depth",
            Self::Bot { depth: false, world: true,  dirac: true  } => "Argmax + world",
            Self::Bot { depth: true,  world: true,  dirac: true  } => "Full subgame solver",
        }
    }
    /// Stable username for DB identity. `bot:<label>`.
    pub fn username(self) -> String {
        format!("bot:{}", self.label())
    }
    /// Deterministic UUID v5 derived from the username.
    pub fn uuid(self) -> uuid::Uuid {
        uuid::Uuid::new_v5(&VARIANT_NAMESPACE, self.username().as_bytes())
    }
    /// True if this variant needs an in-memory blueprint to play.
    pub fn requires_blueprint(self) -> bool {
        matches!(self, Self::Bot { .. })
    }
    /// True if this variant can be chosen as the hero's opponent in a
    /// live game. Currently every variant is spawnable locally except
    /// the slumbot.com adversary which lives outside the zoo.
    pub fn live_spawnable(self) -> bool {
        true
    }
    /// All variants in canonical UI display order: 8 cube cells first
    /// (by axis triple), then fish.
    #[rustfmt::skip]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Bot { depth: false, world: false, dirac: false },
            Self::Bot { depth: true,  world: false, dirac: false },
            Self::Bot { depth: false, world: true,  dirac: false },
            Self::Bot { depth: false, world: false, dirac: true  },
            Self::Bot { depth: true,  world: true,  dirac: false },
            Self::Bot { depth: true,  world: false, dirac: true  },
            Self::Bot { depth: false, world: true,  dirac: true  },
            Self::Bot { depth: true,  world: true,  dirac: true  },
            Self::Fish,
        ]
    }
    /// Parse one CLI / wire token. Returns `None` on unknown tokens;
    /// caller decides between hard-fail and skip.
    pub fn parse(token: &str) -> Option<Self> {
        match token.trim() {
            "fish" => Some(Self::Fish),
            "base" => Some(Self::Bot {
                depth: false,
                world: false,
                dirac: false,
            }),
            other => parse_flags(other),
        }
    }
}

/// Parse a `+`-joined flag token like `depth+dirac` or
/// `depth+world+dirac`. Each flag toggles one axis; flags must appear
/// in canonical order (`depth` < `world` < `dirac`) so a token has
/// exactly one canonical form, matching its label.
fn parse_flags(token: &str) -> Option<Variant> {
    let mut depth = false;
    let mut world = false;
    let mut dirac = false;
    let mut last = 0u8;
    for flag in token.split('+') {
        let pos: u8 = match flag {
            "depth" if !depth => {
                depth = true;
                1
            }
            "world" if !world => {
                world = true;
                2
            }
            "dirac" if !dirac => {
                dirac = true;
                3
            }
            _ => return None,
        };
        if pos <= last {
            return None;
        }
        last = pos;
    }
    Some(Variant::Bot { depth, world, dirac })
}

/// The slumbot.com adversary identity, recorded as a fixed pseudo-bot
/// when the slumbot crate persists hands. Not in the zoo; the slumbot
/// binary plays our zoo bots against this external opponent.
pub fn slumbot_opponent_username() -> &'static str {
    "bot:slumbot"
}

pub fn slumbot_opponent_uuid() -> uuid::Uuid {
    uuid::Uuid::new_v5(&VARIANT_NAMESPACE, slumbot_opponent_username().as_bytes())
}
