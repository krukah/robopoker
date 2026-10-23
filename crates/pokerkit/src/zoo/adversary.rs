//! External opponents we benchmark against — bots that live outside the zoo.
//!
//! A [`Variant`](crate::Variant) is a bot *we* run.

/// A bot outside the zoo, played over its own public API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Adversary {
    /// `slumbot.com` — the long-standing HUNL research bot. Client: `spar`.
    Slumbot,
    /// GTO Wizard AI, via the Ruse researcher API. Client: `ruse`.
    Wizard,
}

impl Adversary {
    /// Canonical service token. Doubles as the DB username suffix, the OTLP metric namespace,
    /// and the `records/` folder name — never change one without the others.
    pub const fn service(self) -> &'static str {
        match self {
            Self::Slumbot => "slumbot",
            Self::Wizard => "gtowizard",
        }
    }

    /// Human-facing name for UI filters and reports.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Slumbot => "slumbot.com",
            Self::Wizard => "GTO Wizard AI",
        }
    }

    /// Stable username for DB identity, matching
    /// [`Variant::username`](crate::Variant::username).
    pub fn username(self) -> String {
        format!("bot:{}", self.service())
    }

    /// Placeholder mailbox, mirroring the zoo's `<label>@robopoker.io`.
    pub fn email(self) -> String {
        format!("{}@robopoker.io", self.service())
    }

    /// Deterministic UUID v5 derived from the username — the join key every persisted hand
    /// already uses.
    pub fn uuid(self) -> uuid::Uuid {
        uuid::Uuid::new_v5(&crate::VARIANT_NAMESPACE, self.username().as_bytes())
    }

    /// Every external opponent, in the order they were added.
    pub const fn all() -> &'static [Self] {
        &[Self::Slumbot, Self::Wizard]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `bot:slumbot` UUID predates this type and is referenced by every hand `spar` has
    /// ever recorded. It must not move.
    #[test]
    fn slumbot_identity_is_frozen() {
        assert_eq!(Adversary::Slumbot.username(), "bot:slumbot");
        assert_eq!(Adversary::Slumbot.uuid(), uuid::Uuid::new_v5(&crate::VARIANT_NAMESPACE, b"bot:slumbot"));
    }

    /// Distinct services never collide in the identity space.
    #[test]
    fn services_are_distinct() {
        assert_ne!(Adversary::Slumbot.uuid(), Adversary::Wizard.uuid());
    }
}
