//! Runtime-dispatched action-translation policy.
//!
//! Mirrors the [`crate::Regime`] pattern: a process-global `OnceLock` set
//! once at startup via [`init_translation`] and queried via [`translation`].
//! Binaries take `--translation <name>` on the CLI via clap.
//!
//! ## Why global, like `Regime`?
//!
//! [`Regime`] selects which abstract tree to train against; [`Translation`]
//! selects how to map an external opponent's off-tree raise onto the
//! abstract tree at inference time. Both are static configuration set
//! at startup — neither changes per-decision.
//!
//! ## Why this does NOT require retraining
//!
//! Training only walks canonical edges; `Game::translate` never
//! observes an off-tree action during training. So all six translations
//! produce identical training output. Only inference (against external
//! opponents who play arbitrary chip amounts) sees the difference.

use rand::Rng;
use rbp_translate::*;

/// Action-translation policy. Each variant names a canonical algorithm;
/// resolve runs that algorithm against a [`Lattice`] and a [`Scalar`]
/// to produce a [`Translated<P, F>`].
///
/// All current variants always return [`Translated::Snap`] (never
/// [`Translated::Free`]) — they snap onto the abstract grid one way or
/// another. Brown-style abstraction-free variants (`Exact`,
/// `EpsilonPrune`, `EpsilonHarmonic`) were elided until a player exists
/// that can consume off-tree resolutions; pure-blueprint players panic
/// on that case.
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
        let _ = free;
        let anchor = match self {
            Self::Snap => lattice.snap(observed),
            Self::Phargmax => lattice.phargmax(observed),
            Self::Harmonic => lattice.harmonic(observed, rng),
        };
        Translated::Snap(*lattice.payload(anchor))
    }
}

impl std::fmt::Display for Translation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Snap => write!(f, "snap"),
            Self::Harmonic => write!(f, "harmonic"),
            Self::Phargmax => write!(f, "phargmax"),
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
        assert_eq!(
            existing, l,
            "translation already set to {:?}, cannot change to {:?}",
            existing, l,
        );
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
        let rng = &mut seeded();
        assert_eq!(
            Translation::Snap.resolve(obs(0.4), &l, (), rng),
            Translated::Snap(())
        );
        assert_eq!(
            Translation::Snap.resolve(obs(0.8), &l, (), rng),
            Translated::Snap(())
        );
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
        let rng = &mut seeded();
        for _ in 0..50 {
            assert_eq!(l.harmonic(obs(0.1), rng), Anchor::new(0));
        }
    }

    #[test]
    fn harmonic_clamps_above() {
        let l = lat([0.5, 1.0, 2.0]);
        let rng = &mut seeded();
        for _ in 0..50 {
            assert_eq!(l.harmonic(obs(10.0), rng), Anchor::new(2));
        }
    }

    #[test]
    fn harmonic_monte_carlo_matches_formula() {
        let l = lat([0.5, 1.0]);
        let rng = &mut seeded();
        let trials = 200_000;
        let lo_hits = (0..trials)
            .filter(|_| l.harmonic(obs(0.75), rng) == Anchor::new(0))
            .count();
        let empirical = lo_hits as f64 / trials as f64;
        let bracket = l.bracket(obs(0.75));
        let expected = l.pharmonic(bracket, obs(0.75));
        assert!(
            (empirical - expected).abs() < 0.005,
            "empirical {empirical} vs expected {expected}",
        );
    }

    #[test]
    fn resolve_returns_payload() {
        let l: Lattice<T, &'static str> = [(0.5, "lo"), (1.0, "mid"), (2.0, "hi")]
            .into_iter()
            .collect();
        let rng = &mut seeded();
        assert_eq!(
            Translation::Snap.resolve(obs(0.4), &l, 0u32, rng),
            Translated::Snap("lo"),
        );
        assert_eq!(
            Translation::Phargmax.resolve(obs(1.9), &l, 0u32, rng),
            Translated::Snap("hi"),
        );
    }
}
