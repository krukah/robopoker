use crate::*;

/// One ranked row: the latest benchmark for a
/// `(regime, version, opponent, variant)` fingerprint, plus that
/// fingerprint's latest litmus score. A fingerprint that has litmus but no
/// benchmark appears with `variant = None` and `bb = None` so it is never
/// silently dropped.
#[derive(serde::Serialize)]
pub struct Standing {
    pub(crate) regime: String,
    pub(crate) version: String,
    pub(crate) fingerprint: String,
    pub(crate) opponent: Option<String>,
    pub(crate) variant: Option<String>,
    pub(crate) epoch: i64,
    pub(crate) hands: i64,
    pub(crate) bb: Option<f64>,
    pub(crate) conf: Option<f64>,
    pub(crate) aivat: Option<f64>,
    pub(crate) litmus: Option<Litscore>,
    pub(crate) stamped: i64,
}
impl Standing {
    /// Render the ranked leaderboard as an aligned text table: `(regime, version,
    /// opponent, variant)` ordered by bb/100 descending, with the opponent's
    /// luck-adjusted rate and litmus pass/total alongside.
    pub fn render(standings: &[Standing]) -> String {
        let header = format!(
            "    {:<9} {:<4} {:<9} {:<12} {:>11} {:>7} {:>8} {:>6} {:>8} {:>8}",
            "regime", "ver", "opponent", "variant", "epoch", "hands", "bb/100", "±95", "aivat", "litmus"
        );
        let rule = "─".repeat(header.chars().count());
        let rows = standings
            .iter()
            .enumerate()
            .map(|(i, s)| {
                format!(
                    "{:>2}. {:<9} {:<4} {:<9} {:<12} {:>11} {:>7} {:>8} {:>6} {:>8} {:>8}",
                    i + 1,
                    s.regime(),
                    s.version(),
                    s.opponent().unwrap_or("—"),
                    s.variant().unwrap_or("—"),
                    s.epoch(),
                    s.hands(),
                    s.bb().map_or_else(|| "—".to_string(), |b| format!("{b:+.1}")),
                    s.conf().map_or_else(|| "—".to_string(), |c| format!("{c:.1}")),
                    s.aivat().map_or_else(|| "—".to_string(), |a| format!("{a:+.1}")),
                    s.litmus()
                        .map_or_else(|| "—".to_string(), |l| format!("{}/{}", l.pass(), l.pass() + l.fail())),
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        format!("{header}\n{rule}\n{rows}")
    }
}

impl Standing {
    fn regime(&self) -> &str {
        &self.regime
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn opponent(&self) -> Option<&str> {
        self.opponent.as_deref()
    }

    fn variant(&self) -> Option<&str> {
        self.variant.as_deref()
    }

    fn epoch(&self) -> i64 {
        self.epoch
    }

    fn hands(&self) -> i64 {
        self.hands
    }

    pub(crate) fn bb(&self) -> Option<f64> {
        self.bb
    }

    fn conf(&self) -> Option<f64> {
        self.conf
    }

    fn aivat(&self) -> Option<f64> {
        self.aivat
    }

    fn litmus(&self) -> Option<&Litscore> {
        self.litmus.as_ref()
    }
}
