//! The behavior half of a [`pokerkit::Regime`].
//!
//! `Regime` is the *selector* — a `clap::ValueEnum` parsed from `--regime`, and the suffix on
//! every policy-layer table. `Regimen` is what the selector resolves to: the bet-size menu for
//! each `(street, depth)` cell, and that menu's identity.
use super::*;
use deuce::Street;

/// A bet-sizing abstraction: which raises exist where, and what that makes it.
pub trait Regimen {
    /// The raise menu for one `(street, depth)` cell, or `None` past
    /// [`pokerkit::MAX_RAISE_REPEATS`], where the raise war is over.
    fn grid(&self, street: Street, depth: usize) -> Option<Grid>;

    /// The sizing terms this regime contributes to the game tree's shape. Joined onto
    /// [`Shape::common`] by [`Self::config`] — this half is the regime's own, and declaring it
    /// is the price of being a regime.
    fn sizing(&self) -> Shape;

    /// Every constant the tree's shape depends on, regime-specific and shared.
    ///
    /// Two runs agree on a blueprint iff they agree on this: it is what
    /// `fingerprint_<regime>_<version>` stores and what the trainer and server compare against
    /// on startup.
    fn config(&self) -> Shape {
        Shape::common().chain(self.sizing())
    }

    /// This regime's game tree, condensed — tags scoreboard rows with the exact tree that
    /// produced them.
    fn digest(&self) -> Digest {
        self.config().digest()
    }
}

impl dyn Regimen {
    /// The active regime's behavior, resolved from the process-global selector. The one place
    /// the enum maps to an implementation.
    pub fn active() -> &'static Self {
        match pokerkit::Regime::current() {
            pokerkit::Regime::Pluribus => &Pluribus,
            pokerkit::Regime::Slumbot => &Slumbot,
        }
    }
}

/// Pluribus-inspired menus (Brown & Sandholm 2019): BB-relative preflop opens, then pot-
/// relative raises keyed on `(street, depth)`.
pub struct Pluribus;

impl Pluribus {
    /// Row into [`pokerkit::PLURIBUS_INDICES`]. Depth ≥ 2 collapses onto the street's "N" row —
    /// a fourth raise gets the same menu as the third.
    const fn row(street: Street, depth: usize) -> usize {
        let street = match street {
            Street::Pref => 0,
            Street::Flop => 3,
            Street::Turn => 6,
            Street::Rive => 9,
        };
        street + if depth > 2 { 2 } else { depth }
    }
}

impl Regimen for Pluribus {
    fn grid(&self, street: Street, depth: usize) -> Option<Grid> {
        if depth > pokerkit::MAX_RAISE_REPEATS {
            None
        } else if matches!(street, Street::Pref) && depth == 0 {
            Some(Grid::Opening(&pokerkit::OPENS))
        } else {
            Some(Grid::Postflop(pokerkit::PLURIBUS_INDICES[Self::row(street, depth)]))
        }
    }

    fn sizing(&self) -> Shape {
        Shape::of([
            ("PLURIBUS_INDICES", format!("{:?}", pokerkit::PLURIBUS_INDICES)),
            ("RAISES", format!("{:?}", pokerkit::RAISES)),
        ])
    }
}

/// Slumbot-matching menu: half pot and full pot at every cell, which minimizes translation loss
/// against that bot's own tree.
pub struct Slumbot;

impl Regimen for Slumbot {
    fn grid(&self, _: Street, depth: usize) -> Option<Grid> {
        (depth <= pokerkit::MAX_RAISE_REPEATS).then_some(Grid::Postflop(pokerkit::SLUMBOT_INDICES))
    }

    fn sizing(&self) -> Shape {
        Shape::of([
            ("SLUMBOT_INDICES", format!("{:?}", pokerkit::SLUMBOT_INDICES)),
            ("RAISES", format!("{:?}", pokerkit::RAISES)),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fingerprint is a wire format, not a debug string: this exact text is what
    /// `fingerprint_pluribus_v1` holds on prod, and the trainer refuses to resume against a
    #[test]
    fn pluribus_config_is_the_stamped_fingerprint() {
        assert_eq!(
            Pluribus.config().to_string(),
            "N=2;STACK=200;B_BLIND=2;S_BLIND=1;MAX_RAISE_REPEATS=3;OPENS=[2, 3, 4, 5];\
             PLURIBUS_INDICES=[[], [5, 8, 9], [5, 8], [0, 2, 5, 8], [2, 5], [5], [1, 2, 5, 8], \
             [5, 8], [5], [1, 2, 5, 8], [5, 8], [5]];\
             RAISES=[(1, 4), (1, 3), (1, 2), (2, 3), (3, 4), (1, 1), (5, 4), (3, 2), (2, 1), (3, 1)]"
        );
    }

    /// Both halves of the config travel together — a regime that forgot its own sizing would
    /// still produce a well-formed string, so assert the shared constants and the regime's own
    #[test]
    fn config_carries_both_halves() {
        let config = Slumbot.config();
        assert!(config.to_string().starts_with(&Shape::common().to_string()));
        assert!(config.terms().iter().any(|(k, _)| *k == "SLUMBOT_INDICES"));
    }

    /// Distinct trees must not share a digest — that is the whole job.
    #[test]
    fn digests_separate_the_regimes() {
        assert_ne!(Pluribus.digest(), Slumbot.digest());
        assert_eq!(Pluribus.digest().to_string().len(), 16);
    }

    /// Past the raise cap there is no menu at all, for either regime.
    #[test]
    fn no_grid_past_the_raise_cap() {
        let past = pokerkit::MAX_RAISE_REPEATS + 1;
        assert_eq!(Pluribus.grid(Street::Flop, past), None);
        assert_eq!(Slumbot.grid(Street::Flop, past), None);
    }

    /// Preflop depth 0 is the one BB-relative cell; everything else is pot-relative. The axis
    /// is the variant, so this is a type-level claim.
    #[test]
    fn opens_are_the_only_bb_relative_cell() {
        assert!(matches!(Pluribus.grid(Street::Pref, 0), Some(Grid::Opening(_))));
        assert!(matches!(Pluribus.grid(Street::Pref, 1), Some(Grid::Postflop(_))));
        assert!(matches!(Pluribus.grid(Street::Flop, 0), Some(Grid::Postflop(_))));
    }
}
