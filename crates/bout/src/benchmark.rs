//! The hand loop, and the statistics it produces.

use crate::mode::*;
use crate::outcome::*;
use crate::recorder::*;
use crate::rival::*;
use crate::stats::*;
use crate::throttle::*;
use daybook::Score;
use daybook::Scoreboard;
use kicker::Turn;
use parlor::VariantExt;
use pokerkit::Adversary;
use pokerkit::Variant;
use tracing::Instrument;
use vitals::KeyValue;

/// One variant's run against one external opponent.
///
/// The loop is deliberately forgiving: a hand that fails is counted, logged, and dropped, and
/// the session reconnects. Against a research API we don't control, an occasional 5xx or a spot
/// our translation mishandles must not end a 48-hour run — but it must be visible, which is why
/// errors are their own counter rather than a silent `continue`.
pub struct Benchmark {
    rival: Adversary,
    variant: Variant,
    hands: Vec<Outcome>,
}

/// How often to log a progress line, in hands.
const CADENCE: usize = 100;

impl Benchmark {
    /// Play until `mode` says stop, hero's seat alternating however the
    /// service deals it.
    pub async fn run<R>(
        auth: &R::Auth,
        variant: Variant,
        player: &mut dyn parlor::Player,
        recorder: &mut Recorder,
        throttle: Throttle,
        mode: Mode,
    ) -> Self
    where
        R: Rival,
    {
        let mut bench = Self {
            rival: R::ADVERSARY,
            variant,
            hands: Vec::new(),
        };
        let mut rival = R::fresh(auth, throttle.clone());
        let mut errors = 0u32;
        while !pokerkit::Signals::interrupted() && !mode.done(bench.hands().len()) {
            let span = tracing::info_span!(
                "bout.hand",
                rival = R::ADVERSARY.service(),
                variant = variant.label(),
                index = bench.hands().len()
            );
            match rival.play(player, recorder).instrument(span).await {
                Ok(outcome) => {
                    errors = 0;
                    bench.push(outcome);
                    tokio::time::sleep(mode.pace(R::ADVERSARY)).await;
                }
                Err(e) => {
                    errors += 1;
                    bench.fault(e, errors, mode);
                    rival = R::fresh(auth, throttle.clone());
                    tokio::time::sleep(mode.backoff(errors)).await;
                }
            }
        }
        tracing::info!(
            rival = bench.rival().service(),
            variant = variant.label(),
            played = bench.hands().len(),
            target = ?mode.target(),
            "bout run finished",
        );
        if let Some(audit) = rival.audit().await {
            tracing::info!(
                rival = bench.rival().service(),
                variant = variant.label(),
                audit,
                "opponent's own scorecard"
            );
        }
        bench
    }

    fn rival(&self) -> Adversary {
        self.rival
    }

    fn variant(&self) -> Variant {
        self.variant
    }

    fn hands(&self) -> &[Outcome] {
        &self.hands
    }

    fn hands_mut(&mut self) -> &mut Vec<Outcome> {
        &mut self.hands
    }

    /// Accept a finished hand: metrics, tally, and a periodic progress line.
    fn push(&mut self, outcome: Outcome) {
        let labels = self.labels();
        let family = self.family();
        family.hands().add(1, &labels);
        family.hand_bb().record(outcome.chips(), &labels);
        if outcome.chips() >= 0.0 {
            family.hand_bb_won().add(outcome.chips(), &labels);
        } else {
            family.hand_bb_lost().add(-outcome.chips(), &labels);
        }
        self.hands_mut().push(outcome);
        if self.hands().len().is_multiple_of(CADENCE) {
            tracing::info!(
                rival = self.rival().service(),
                variant = self.variant().label(),
                played = self.hands().len(),
                bb_per_100 = self.chips().rate(),
                total_bb = self.chips().total(),
                "bout progress",
            );
        }
    }

    /// Note a failed hand. `errors` is the consecutive-failure count, which
    /// drives the backoff and tells an operator whether this is noise or an
    /// outage.
    fn fault(&self, error: anyhow::Error, errors: u32, mode: Mode) {
        self.family().errors().add(1, &self.labels());
        tracing::warn!(
            rival = self.rival().service(),
            variant = self.variant().label(),
            hand = self.hands().len() + 1,
            retry = errors,
            backoff_s = mode.backoff(errors).as_secs(),
            error = %error,
            "bout hand failed",
        );
    }

    /// This service's OTLP metric family — `rbp.<service>.*`.
    fn family(&self) -> &'static vitals::Bench {
        let handles = vitals::metrics::Handles::get();
        match self.rival() {
            Adversary::Slumbot => handles.slumbot(),
            Adversary::Wizard => handles.gtowizard(),
        }
    }

    /// Cube coordinate (4 keys) plus regime. The opponent is already in the
    /// metric name, so it stays out of the label set.
    fn labels(&self) -> [KeyValue; 5] {
        let [v, p, d, w] = self.variant().keys();
        [
            v,
            p,
            d,
            w,
            KeyValue::new("regime", pokerkit::Regime::current().to_string()),
        ]
    }

    /// Chips actually won, over every hand.
    fn chips(&self) -> Stats {
        Stats::of(self.hands().iter().map(Outcome::chips))
    }

    /// Chips won from one seat only — the button and the big blind have
    /// very different expectations, and a lopsided split hides a bug.
    fn seat(&self, hero: Turn) -> Stats {
        Stats::of(self.hands().iter().filter(|h| h.hero() == hero).map(Outcome::chips))
    }

    /// The opponent's luck-adjusted score, over the hands it reported one
    /// for. Empty where the service doesn't compute AIVAT.
    fn aivat(&self) -> Stats {
        Stats::of(self.hands().iter().filter_map(Outcome::aivat))
    }

    /// Log the run's headline figures.
    pub fn report(&self) {
        let chips = self.chips();
        let aivat = self.aivat();
        tracing::info!(
            rival = self.rival().service(),
            variant = self.variant().label(),
            hands = chips.hands(),
            sb_hands = self.seat(Turn::Choice(0)).hands(),
            bb_hands = self.seat(Turn::Choice(1)).hands(),
            total_bb = chips.total(),
            bb_per_100 = chips.rate(),
            confidence = chips.confidence(),
            stddev = chips.stddev(),
            bb_per_100_as_sb = self.seat(Turn::Choice(0)).rate(),
            bb_per_100_as_bb = self.seat(Turn::Choice(1)).rate(),
            aivat_hands = aivat.hands(),
            aivat_per_100 = aivat.rate(),
            aivat_lower = aivat.lower(),
            "bout benchmark complete",
        );
    }

    /// Append this run's aggregate to the shared `benchmark` table, tagged
    /// with the active fingerprint and the blueprint `epoch` it played, so it
    /// ranks against every other config.
    pub async fn persist(&self, db: &tokio_postgres::Client, epoch: i64) {
        let chips = self.chips();
        let aivat = self.aivat();
        db.record_benchmark(
            &Score::new(
                self.rival().service(),
                self.variant().label(),
                epoch,
                chips.hands() as i64,
                chips.rate(),
                chips.confidence(),
                chips.stddev(),
            )
            .with_aivat((aivat.hands() > 0).then(|| aivat.rate()), (aivat.hands() > 0).then(|| aivat.confidence())),
        )
        .await;
    }
}
