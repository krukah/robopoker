//! Sum-friendly CPU accounting for Rayon tasks (`cpu-time`, not wall clock).

use std::time::Duration;

/// Runs `f`, returning its output plus this OS thread's consumed CPU time for that call.
///
/// On non-Unix targets returns [`Duration::ZERO`] for the timing arm (placeholder).
pub(crate) fn measure<R>(f: impl FnOnce() -> R) -> (R, Duration) {
    #[cfg(unix)]
    {
        let start = timespec_now_thread_cpu();
        let out = f();
        let elapsed = timespec_duration_since(&start);
        (out, elapsed)
    }
    #[cfg(not(unix))]
    {
        (f(), Duration::ZERO)
    }
}

#[cfg(unix)]
fn timespec_now_thread_cpu() -> libc::timespec {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `ts` is a valid out-parameter per POSIX / Darwin.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) };
    debug_assert_eq!(rc, 0);
    ts
}

#[cfg(unix)]
fn timespec_duration_since(start: &libc::timespec) -> Duration {
    let end = timespec_now_thread_cpu();
    let s_ns =
        i128::from(start.tv_sec).saturating_mul(1_000_000_000) + i128::from(start.tv_nsec);
    let e_ns = i128::from(end.tv_sec).saturating_mul(1_000_000_000) + i128::from(end.tv_nsec);
    let delta_ns = (e_ns - s_ns).max(0).min(i128::from(u64::MAX));
    Duration::from_nanos(delta_ns as u64)
}
