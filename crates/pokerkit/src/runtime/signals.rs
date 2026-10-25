#![allow(dead_code)]

use super::*;

/// Process-wide interrupt handling and duration parsing.
pub struct Signals;

impl Signals {
    /// Table-name component that isolates training-derived (`regime!`) tables by player count, so a
    /// compile-time `N` change can never address another game's
    /// blueprint/epoch/staging/snapshot/fingerprint tables regardless of the runtime regime.
    pub const fn players_suffix() -> &'static str {
        match N {
            2 => "",
            6 => "_6max",
            _ => panic!("add a players_suffix arm for this N"),
        }
    }

    const fn pick<const N: usize>(idx: [usize; N]) -> [(Chips, Chips); N] {
        let mut r = [(0, 0); N];
        let mut i = 0;
        while i < N {
            r[i] = RAISES[idx[i]];
            i += 1;
        }
        r
    }

    /// Register Ctrl+C handler for immediate (non-graceful) termination.
    #[cfg(feature = "server")]
    pub fn kys() {
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            println!();
            tracing::warn!("violent interrupt received, exiting immediately");
            std::process::exit(0);
        });
    }

    /// Check if graceful shutdown was requested (via stdin "Q") or deadline reached.
    #[cfg(feature = "server")]
    pub fn interrupted() -> bool {
        INTERRUPTED.load(std::sync::atomic::Ordering::Relaxed)
            || DEADLINE.get().is_some_and(|d| std::time::Instant::now() >= *d)
    }

    /// No-op interrupt check when server feature disabled.
    #[cfg(not(feature = "server"))]
    pub fn interrupted() -> bool {
        false
    }

    /// Register graceful interrupt handler. Type "Q" + Enter to stop after current batch. Also
    /// handles SIGTERM (sent by ECS stop-task) for graceful remote shutdown. Optionally set
    /// TRAIN_DURATION env var (e.g., "2h", "30m") for timed runs.
    #[cfg(feature = "server")]
    pub fn brb() {
        std::env::var("TRAIN_DURATION")
            .ok()
            .and_then(|d| Signals::parse_duration(&d).map(|dur| (d, dur)))
            .inspect(|(s, dur)| {
                let _ = DEADLINE.set(std::time::Instant::now() + *dur);
                tracing::info!(duration = %s, "training will stop after duration");
            });
        tokio::spawn(async move {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to register SIGTERM handler")
                .recv()
                .await;
            tracing::warn!("SIGTERM received, finishing current batch...");
            INTERRUPTED.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        std::thread::spawn(|| {
            // A detached process reads EOF (Ok(0)) forever; stop listening rather than spin.
            let ref mut buffer = String::new();
            while std::io::stdin().read_line(buffer).is_ok_and(|n| n > 0) {
                if buffer.trim().eq_ignore_ascii_case("Q") {
                    tracing::warn!("graceful interrupt requested, finishing current batch...");
                    INTERRUPTED.store(true, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
                buffer.clear();
            }
        });
    }

    /// Parse duration string like "30s", "5m", "2h", "1d" into Duration.
    #[cfg(feature = "server")]
    fn parse_duration(s: &str) -> Option<std::time::Duration> {
        let s = s.trim();
        let (num, unit) = s.split_at(s.len().saturating_sub(1));
        let value: u64 = num
            .parse()
            .inspect_err(|e| tracing::warn!(input = %num, error = %e, "parse_duration: number parse failed"))
            .ok()?;
        match unit {
            "s" => Some(std::time::Duration::from_secs(value)),
            "m" => Some(std::time::Duration::from_secs(value * 60)),
            "h" => Some(std::time::Duration::from_secs(value * 3600)),
            "d" => Some(std::time::Duration::from_secs(value * 86400)),
            _ => None,
        }
    }
}
