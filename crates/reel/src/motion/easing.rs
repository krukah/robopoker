use super::*;
use pokerkit::Utility;

/// Easing and layout helpers for the film.
pub(crate) struct Easing;
impl Easing {
    /// Hold, then rise smoothly to a phase's new state — the shared crossfade
    /// of every panel on the reel clock.
    pub fn fade(t: f64) -> f64 {
        let r = (t / RISE).clamp(0.0, 1.0);
        r * r * (3.0 - 2.0 * r)
    }

    /// The shared reel clock: SPAN ticks per phase then the linger, as
    /// (phase, t) pairs every panel maps its state over.
    pub fn ticks(phases: usize) -> impl Iterator<Item = (usize, f64)> {
        (0..phases)
            .flat_map(|s| (0..SPAN).map(move |f| (s, f as f64 / SPAN as f64)))
            .chain((0..LINGER).map(move |_| (phases - 1, 1.0)))
    }

    /// Regret magnitude as stroke weight — the loudest visual lever.
    pub(crate) fn breadth(gain: Utility) -> f64 {
        WIDTH.0 + f64::from(gain.abs().min(1.0)) * WIDTH.1
    }

    /// A rect expanded uniformly by `pad` on every side — the rim offset.
    pub(crate) fn grow(rect: Rect, pad: f64) -> Rect {
        Rect::new(rect.x() - pad, rect.y() - pad, rect.w() + 2.0 * pad, rect.h() + 2.0 * pad)
    }
}
