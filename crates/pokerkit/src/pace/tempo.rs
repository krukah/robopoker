/// How fast a hand plays.
///
/// Every paced beat is the same base duration scaled by one number, so the
/// table's rhythm is a single choice rather than six that have to be kept in
/// proportion by hand. A human's decision window is deliberately *not*
/// scaled: that is the player's clock, not the table's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Tempo {
    Brisk,
    #[default]
    Table,
    Slow,
}

impl Tempo {
    /// Thousandths of the base pace. Integer math so the whole scale stays
    /// `const` and no beat can drift by a rounding mode.
    pub const fn permille(self) -> u64 {
        match self {
            Self::Brisk => 500,
            Self::Table => 1000,
            Self::Slow => 1800,
        }
    }

    pub const fn scale(self, base: u64) -> u64 {
        base * self.permille() / 1000
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Brisk => "Brisk",
            Self::Table => "Table",
            Self::Slow => "Slow",
        }
    }

    pub const fn all() -> [Self; 3] {
        [Self::Brisk, Self::Table, Self::Slow]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_tempo_is_the_base_pace() {
        assert_eq!(Tempo::default().scale(1000), 1000);
    }

    #[test]
    fn tempo_orders_the_beats_it_scales() {
        assert!(Tempo::Brisk.scale(1000) < Tempo::Table.scale(1000));
        assert!(Tempo::Table.scale(1000) < Tempo::Slow.scale(1000));
    }

    #[test]
    fn a_zero_beat_stays_zero_at_every_tempo() {
        assert!(Tempo::all().into_iter().all(|t| t.scale(0) == 0));
    }
}
