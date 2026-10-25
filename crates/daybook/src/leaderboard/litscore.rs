/// Litmus pass/fail/skip/error counts for one fingerprint.
#[derive(Clone, serde::Serialize)]
pub struct Litscore {
    pub(crate) pass: i64,
    pub(crate) fail: i64,
    pub(crate) skip: i64,
    pub(crate) error: i64,
}
impl Litscore {
    /// Fraction of decisive (pass + fail) cases that passed. Skips and
    /// errors are excluded — they measure catalog coverage, not quality.
    pub fn rate(&self) -> f64 {
        let decisive = self.pass() + self.fail();
        if decisive == 0 { 0.0 } else { self.pass() as f64 / decisive as f64 }
    }
}

impl Litscore {
    pub(crate) fn pass(&self) -> i64 {
        self.pass
    }

    pub(crate) fn fail(&self) -> i64 {
        self.fail
    }
}
