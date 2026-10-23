use opentelemetry::metrics::Counter;
use opentelemetry::metrics::Histogram;
use opentelemetry::metrics::Meter;

/// One benchmark service's metric family, registered under
/// `rbp.<service>.*`. The shape is identical for every opponent we play, so
/// it is declared once and built per service rather than copied per name —
/// a new adversary is one line in `build`, not five.
#[rustfmt::skip]
pub struct Bench {
    hands: Counter   <u64>,
    hand_bb_won: Counter   <f64>,
    hand_bb_lost: Counter   <f64>,
    errors: Counter   <u64>,
    hand_bb: Histogram <f64>,
}
impl Bench {
    /// `service` is the canonical token from `pokerkit::Adversary::service`;
    /// it is the only thing that varies between families.
    pub(crate) fn build(meter: &Meter, service: &str) -> Self {
        Self {
            hands: meter
                .u64_counter(format!("rbp.{service}.hands"))
                .with_description("Benchmark hands played against this opponent")
                .build(),
            hand_bb: meter
                .f64_histogram(format!("rbp.{service}.hand_bb"))
                .with_description("Winnings per hand in big blinds (signed; use _bucket for distribution, not _sum)")
                .build(),
            hand_bb_won: meter
                .f64_counter(format!("rbp.{service}.hand_bb_won"))
                .with_description(
                    "Cumulative big blinds won (positive side only). Paired with \
                     hand_bb_lost, the signed sum is `won - lost`; unlike a histogram's \
                     _sum this pair is monotonic so Prometheus rate()/increase() is safe.",
                )
                .build(),
            hand_bb_lost: meter
                .f64_counter(format!("rbp.{service}.hand_bb_lost"))
                .with_description(
                    "Cumulative big blinds lost (absolute value of negative hand results). \
                     See hand_bb_won for mean-bb/100 computation.",
                )
                .build(),
            errors: meter
                .u64_counter(format!("rbp.{service}.errors"))
                .with_description("Benchmark session failures against this opponent")
                .build(),
        }
    }
}
impl Bench {
    pub fn hands(&self) -> &Counter<u64> {
        &self.hands
    }

    pub fn hand_bb_won(&self) -> &Counter<f64> {
        &self.hand_bb_won
    }

    pub fn hand_bb_lost(&self) -> &Counter<f64> {
        &self.hand_bb_lost
    }

    pub fn errors(&self) -> &Counter<u64> {
        &self.errors
    }

    pub fn hand_bb(&self) -> &Histogram<f64> {
        &self.hand_bb
    }
}
