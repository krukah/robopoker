//! Sum-friendly CPU accounting for parallel batches (CPU time, not wall clock).
//!
//! Once [`Solver::batch`](crate::Solver::batch) fans trees out across Rayon
//! workers, wall-clock time no longer reflects work done. [`measure`] returns a
//! closure's result plus the calling OS thread's CPU time for that call; summed
//! across workers it gives total CPU-seconds per batch, which divided by
//! (wall-time × worker count) is the CPU-utilization figure that motivated the
//! parallel batch design.
//!
//! Ported from the technique in robopoker PR #54 (andyafter). The parallel
//! batch itself already lived in `Solver::batch`; this adds the accounting.

use std::time::Duration;

/// Runs `f`, returning its output plus this OS thread's CPU time consumed by it.
/// Returns [`Duration::ZERO`] for the timing on non-Unix targets (placeholder).
pub(crate) fn measure<R>(f: impl FnOnce() -> R) -> (R, Duration) {
    #[cfg(unix)]
    {
        let start = now();
        let out = f();
        (out, since(start))
    }
    #[cfg(not(unix))]
    {
        (f(), Duration::ZERO)
    }
}

#[cfg(unix)]
fn now() -> libc::timespec {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: `ts` is a valid out-parameter for clock_gettime per POSIX / Darwin.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &raw mut ts) };
    debug_assert_eq!(rc, 0);
    ts
}

#[cfg(unix)]
fn since(start: libc::timespec) -> Duration {
    let end = now();
    let ns = |t: &libc::timespec| i128::from(t.tv_sec) * 1_000_000_000 + i128::from(t.tv_nsec);
    Duration::from_nanos((ns(&end) - ns(&start)).clamp(0, i128::from(u64::MAX)) as u64)
}
