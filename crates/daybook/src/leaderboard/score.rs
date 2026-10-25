/// One finished benchmark run, ready to append to the scoreboard.
///
/// Holds what the caller measured; `regime` / `version` / `fingerprint` are filled from process
/// globals at insert time because they describe the binary that did the measuring, not the run.
/// `aivat` is populated only by opponents that compute a luck-adjusted score for us — see
/// [`crate::Scoreboard::record_benchmark`].
pub struct Score {
    opponent: &'static str,
    variant: &'static str,
    epoch: i64,
    hands: i64,
    bb: f64,
    conf: f64,
    stddev: f64,
    aivat: Option<f64>,
    aconf: Option<f64>,
    stamped: i64,
}
impl Score {
    /// A run scored by the chips that actually moved. Stamped now — the
    /// timestamp is what makes this row the latest for its key.
    pub fn new(
        opponent: &'static str,
        variant: &'static str,
        epoch: i64,
        hands: i64,
        bb: f64,
        conf: f64,
        stddev: f64,
    ) -> Self {
        Self {
            opponent,
            variant,
            epoch,
            hands,
            bb,
            conf,
            stddev,
            aivat: None,
            aconf: None,
            stamped: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock before the epoch")
                .as_secs() as i64,
        }
    }

    /// Attach the opponent's luck-adjusted rate and its 95% half-width, both
    /// in bb/100.
    pub fn with_aivat(self, aivat: Option<f64>, aconf: Option<f64>) -> Self {
        Self { aivat, aconf, ..self }
    }
}

impl Score {
    pub fn opponent(&self) -> &&'static str {
        &self.opponent
    }

    pub fn variant(&self) -> &&'static str {
        &self.variant
    }

    pub fn epoch(&self) -> i64 {
        self.epoch
    }

    pub fn hands(&self) -> i64 {
        self.hands
    }

    pub fn bb(&self) -> f64 {
        self.bb
    }

    pub fn conf(&self) -> f64 {
        self.conf
    }

    pub fn stddev(&self) -> f64 {
        self.stddev
    }

    pub fn aivat(&self) -> &Option<f64> {
        &self.aivat
    }

    pub fn aconf(&self) -> &Option<f64> {
        &self.aconf
    }

    pub fn stamped(&self) -> i64 {
        self.stamped
    }
}
