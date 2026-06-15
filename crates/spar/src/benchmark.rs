use crate::client::*;
use crate::recorder::*;
use crate::result::*;
use crate::session::*;
use kicker::Turn;
use parlor::VariantExt;
use pokerkit::Variant;
use tracing::Instrument;
use vitals::KeyValue;

pub struct Benchmark {
    results: Vec<HandResult>,
}

impl Benchmark {
    /// Play N hands against Slumbot, logging progress periodically.
    ///
    /// `throttle` is shared across all concurrently-running benchmarks so
    /// the aggregate in-flight cap is enforced globally, not per-variant.
    pub async fn run(
        variant: Variant,
        player: &mut dyn parlor::Player,
        hands: usize,
        recorder: &mut Recorder,
        throttle: Throttle,
    ) -> anyhow::Result<Self> {
        let label = variant.label();
        let mut bench = Self { results: Vec::new() };
        let mut client = Client::new().with_throttle(throttle.clone());
        for i in 0..hands {
            if pokerkit::interrupted() {
                tracing::info!(
                    variant = label,
                    played = bench.results.len(),
                    total = hands,
                    "slumbot interrupted (TRAIN_DURATION elapsed or SIGTERM received)",
                );
                break;
            }
            let span = tracing::info_span!("slumbot.hand", variant = label, index = i);
            let outcome = Session::play(&mut client, player, recorder).instrument(span).await;
            match outcome {
                Ok(result) => {
                    record_hand(variant, &result);
                    bench.results.push(result);
                    if (i + 1) % 100 == 0 || i + 1 == hands {
                        tracing::info!(
                            variant = label,
                            played = i + 1,
                            total = hands,
                            bb_per_100 = bench.bb_per_100(),
                            total_bb = bench.total_bb(),
                            "slumbot progress",
                        );
                    }
                }
                Err(e) => {
                    record_error(variant);
                    tracing::warn!(variant = label, hand = i + 1, error = %e, "slumbot hand failed");
                    client = Client::new().with_throttle(throttle.clone());
                }
            }
        }
        Ok(bench)
    }
    /// Play hands continuously until interrupted, with rate limiting and backoff.
    pub async fn continuous(
        variant: Variant,
        player: &mut dyn parlor::Player,
        recorder: &mut Recorder,
        throttle: Throttle,
    ) -> Self {
        let label = variant.label();
        let delay = std::env::var("SLUMBOT_DELAY_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(500u64);
        let mut bench = Self { results: Vec::new() };
        let mut client = Client::new().with_throttle(throttle.clone());
        let mut errors = 0u32;
        loop {
            if pokerkit::interrupted() {
                tracing::info!(variant = label, hands = bench.results.len(), "slumbot interrupted");
                break;
            }
            let span = tracing::info_span!("slumbot.hand", variant = label, index = bench.results.len());
            let outcome = Session::play(&mut client, player, recorder).instrument(span).await;
            match outcome {
                Ok(result) => {
                    record_hand(variant, &result);
                    bench.results.push(result);
                    errors = 0;
                    if bench.results.len().is_multiple_of(100) {
                        tracing::info!(
                            variant = label,
                            played = bench.results.len(),
                            last_hand_bb = bench.results.last().unwrap().winnings_bb,
                            bb_per_100 = bench.bb_per_100(),
                            "slumbot progress",
                        );
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
                Err(e) => {
                    record_error(variant);
                    errors += 1;
                    tracing::warn!(
                        variant = label,
                        hand = bench.results.len() + 1,
                        retry = errors,
                        error = %e,
                        "slumbot hand failed",
                    );
                    client = Client::new().with_throttle(throttle.clone());
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(errors.min(8)).min(300))).await;
                }
            }
        }
        bench
    }

    fn total_bb(&self) -> f64 {
        self.results.iter().map(|r| r.winnings_bb).sum()
    }

    fn bb_per_100(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.total_bb() / self.results.len() as f64 * 100.0
    }

    fn stddev(&self) -> f64 {
        if self.results.len() < 2 {
            return 0.0;
        }
        let mean = self.total_bb() / self.results.len() as f64;
        (self.results.iter().map(|r| (r.winnings_bb - mean).powi(2)).sum::<f64>() / (self.results.len() - 1) as f64)
            .sqrt()
    }

    fn confidence(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        1.96 * self.stddev() / (self.results.len() as f64).sqrt() * 100.0
    }

    fn bb_per_100_at(&self, hero: Turn) -> f64 {
        let (n, sum) = self
            .results
            .iter()
            .filter(|r| r.hero == hero)
            .fold((0usize, 0.0f64), |(n, s), r| (n + 1, s + r.winnings_bb));
        if n == 0 { 0.0 } else { sum / n as f64 * 100.0 }
    }

    pub fn report(&self) {
        let bb = self.results.iter().filter(|r| r.hero == Turn::Choice(1)).count();
        let sb = self.results.iter().filter(|r| r.hero == Turn::Choice(0)).count();
        tracing::info!(
            hands = self.results.len(),
            bb_hero = bb,
            sb_hero = sb,
            total_bb = self.total_bb(),
            bb_per_100 = self.bb_per_100(),
            confidence = self.confidence(),
            bb_per_100_as_bb = self.bb_per_100_at(Turn::Choice(1)),
            bb_per_100_as_sb = self.bb_per_100_at(Turn::Choice(0)),
            stddev = self.stddev(),
            "slumbot benchmark complete",
        );
    }
}

/// Standard slumbot metric label set: cube coordinate (4 keys) + regime.
fn labels(variant: Variant) -> [KeyValue; 5] {
    let [v, p, d, w] = variant.keys();
    [v, p, d, w, KeyValue::new("regime", pokerkit::regime().to_string())]
}

fn record_hand(variant: Variant, result: &HandResult) {
    let lab = labels(variant);
    let m = vitals::metrics::get();
    m.slumbot_hands.add(1, &lab);
    m.slumbot_hand_bb.record(result.winnings_bb, &lab);
    if result.winnings_bb >= 0.0 {
        m.slumbot_hand_bb_won.add(result.winnings_bb, &lab);
    } else {
        m.slumbot_hand_bb_lost.add(-result.winnings_bb, &lab);
    }
}

fn record_error(variant: Variant) {
    let lab = labels(variant);
    vitals::metrics::get().slumbot_errors.add(1, &lab);
}
