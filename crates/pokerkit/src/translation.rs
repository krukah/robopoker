//! Runtime-dispatched action-translation policy.
//!
//! Global `OnceLock` like [`crate::Regime`], set at startup via
//! [`init_translation`] (`--translation <name>` on the CLI). `Regime` selects
//! which abstract tree to train against; `Translation` selects how to map an
//! external opponent's off-tree raise onto that tree at inference time. Both
//! are static config — neither changes per-decision.
//!
//! Changing it does NOT require retraining: training only walks canonical
//! edges, so `Game::translate` never sees an off-tree action and every
//! translation yields identical training output. Only inference against
//! external opponents (arbitrary chip amounts) sees the difference.

use crate::translate::*;
use rand::Rng;

/// Action-translation policy. Each variant names a canonical algorithm;
/// resolve runs that algorithm against a [`Lattice`] and a [`Scalar`]
/// to produce a [`Translated<P, F>`].
///
/// The snap-family variants (`Snap`, `Harmonic`, `Phargmax`) always
/// return [`Translated::Snap`] — they map an off-grid raise onto the
/// abstract grid one way or another. The Brown-style [`Self::Exact`]
/// variant instead returns [`Translated::Free`] for genuinely off-grid
/// raises, deferring them to a nesting player that re-solves an
/// augmented subgame (see `docs/active/off-tree-nesting.md`). A
/// pure-blueprint player must not run under `Exact` — it has no way to
/// consume the off-tree resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Translation {
    /// Deterministic L1 nearest. Reproduces the pre-existing `Game::edgify` snap.
    #[default]
    Snap,
    /// Randomized pseudo-harmonic mapping (Ganzfried-Sandholm 2013).
    Harmonic,
    /// Deterministic argmax variant of [`Self::Harmonic`] for replay.
    Phargmax,
    /// Brown-style abstraction-free (Modicum 2018). Snaps only when the
    /// raise lands exactly on a grid anchor; otherwise emits
    /// [`Translated::Free`] carrying the caller's off-grid value.
    Exact,
}

impl Translation {
    /// Resolve `observed` against `lattice` end-to-end. The `free` value
    /// is the off-grid representation the caller will receive back via
    /// [`Translated::Free`] when a Brown-style variant elects not to
    /// snap. Today's variants never read it; it is moved unconditionally
    /// to keep the call site shape stable across future variants.
    pub fn resolve<A, P, F, R>(
        &self,
        observed: Scalar<A>,
        lattice: &Lattice<A, P>,
        free: F,
        rng: &mut R,
    ) -> Translated<P, F>
    where
        A: Axis,
        P: Copy,
        R: Rng + ?Sized,
    {
        match self {
            Self::Snap => Translated::Snap(*lattice.payload(lattice.snap(observed))),
            Self::Phargmax => Translated::Snap(*lattice.payload(lattice.phargmax(observed))),
            Self::Harmonic => Translated::Snap(*lattice.payload(lattice.harmonic(observed, rng))),
            Self::Exact => match lattice.exact(observed) {
                Some(anchor) => Translated::Snap(*lattice.payload(anchor)),
                None => Translated::Free(free),
            },
        }
    }
}

impl std::fmt::Display for Translation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Snap => write!(f, "snap"),
            Self::Harmonic => write!(f, "harmonic"),
            Self::Phargmax => write!(f, "phargmax"),
            Self::Exact => write!(f, "exact"),
        }
    }
}

static TRANSLATION: std::sync::OnceLock<Translation> = std::sync::OnceLock::<Translation>::new();

/// Returns the active translation. Defaults to `Snap` if [`init_translation`]
/// was never called. Binaries should require explicit `--translation` via clap.
pub fn translation() -> Translation {
    *TRANSLATION.get_or_init(|| Translation::Snap)
}

/// Sets the active translation. Must be called before any inference path
/// queries it. Panics if called twice with different values.
pub fn init_translation(l: Translation) {
    if let Err(existing) = TRANSLATION.set(l) {
        assert_eq!(existing, l, "translation already set to {existing:?}, cannot change to {l:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    struct T;
    impl Axis for T {}

    fn obs(x: f64) -> Scalar<T> {
        Scalar::new(x)
    }

    fn lat(xs: impl IntoIterator<Item = f64>) -> Lattice<T> {
        xs.into_iter().collect()
    }

    fn seeded() -> SmallRng {
        SmallRng::seed_from_u64(0xDEADBEEF)
    }

    #[test]
    fn snap_picks_nearest() {
        let l = lat([0.5, 1.0, 2.0]);
        let ref mut rng = seeded();
        assert_eq!(Translation::Snap.resolve(obs(0.4), &l, (), rng), Translated::Snap(()));
        assert_eq!(Translation::Snap.resolve(obs(0.8), &l, (), rng), Translated::Snap(()));
        assert_eq!(l.snap(obs(0.4)), Anchor::new(0));
        assert_eq!(l.snap(obs(0.8)), Anchor::new(1));
        assert_eq!(l.snap(obs(3.0)), Anchor::new(2));
    }

    #[test]
    fn harmonic_argmax_deterministic() {
        let l = lat([0.5, 1.0]);
        assert_eq!(l.phargmax(obs(0.55)), Anchor::new(0));
        assert_eq!(l.phargmax(obs(0.95)), Anchor::new(1));
    }

    #[test]
    fn harmonic_clamps_below() {
        let l = lat([0.5, 1.0, 2.0]);
        let ref mut rng = seeded();
        for _ in 0..50 {
            assert_eq!(l.harmonic(obs(0.1), rng), Anchor::new(0));
        }
    }

    #[test]
    fn harmonic_clamps_above() {
        let l = lat([0.5, 1.0, 2.0]);
        let ref mut rng = seeded();
        for _ in 0..50 {
            assert_eq!(l.harmonic(obs(10.0), rng), Anchor::new(2));
        }
    }

    #[test]
    fn harmonic_monte_carlo_matches_formula() {
        let l = lat([0.5, 1.0]);
        let ref mut rng = seeded();
        let trials = 200_000;
        let lo_hits = (0..trials)
            .filter(|_| l.harmonic(obs(0.75), rng) == Anchor::new(0))
            .count();
        let empirical = lo_hits as f64 / trials as f64;
        let bracket = l.bracket(obs(0.75));
        let expected = l.pharmonic(bracket, obs(0.75));
        assert!((empirical - expected).abs() < 0.005, "empirical {empirical} vs expected {expected}");
    }

    #[test]
    fn resolve_returns_payload() {
        let l: Lattice<T, &'static str> = [(0.5, "lo"), (1.0, "mid"), (2.0, "hi")].into_iter().collect();
        let ref mut rng = seeded();
        assert_eq!(Translation::Snap.resolve(obs(0.4), &l, 0u32, rng), Translated::Snap("lo"),);
        assert_eq!(Translation::Phargmax.resolve(obs(1.9), &l, 0u32, rng), Translated::Snap("hi"),);
    }

    #[test]
    fn exact_snaps_on_anchor_only() {
        let l: Lattice<T, &'static str> = [(0.5, "lo"), (1.0, "mid"), (2.0, "hi")].into_iter().collect();
        let ref mut rng = seeded();
        assert_eq!(Translation::Exact.resolve(obs(1.0), &l, 42u32, rng), Translated::Snap("mid"));
        assert_eq!(Translation::Exact.resolve(obs(0.5), &l, 42u32, rng), Translated::Snap("lo"));
        assert_eq!(Translation::Exact.resolve(obs(0.7), &l, 42u32, rng), Translated::Free(42u32));
        assert_eq!(Translation::Exact.resolve(obs(5.0), &l, 42u32, rng), Translated::Free(42u32));
    }

    #[test]
    fn exact_snaps_within_relative_tolerance() {
        let l = lat([0.5, 1.0, 2.0]);
        // exact hit (plus float noise) on the 1.0 anchor
        assert_eq!(l.exact(obs(1.0 / 3.0 + 2.0 / 3.0)), Some(Anchor::new(1)));
        // within the 5% relative band of 1.0 (+4%) → snaps to the anchor
        assert_eq!(l.exact(obs(1.04)), Some(Anchor::new(1)));
        // beyond the band of every anchor (20% off 1.0) → off-grid
        assert_eq!(l.exact(obs(1.2)), None);
    }
}
