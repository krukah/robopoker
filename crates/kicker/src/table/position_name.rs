/// Named position at a poker table relative to the dealer button.
///
/// Position names vary by table size:
/// - Heads-up (2): BTN (=SB), BB
/// - 6-max: BTN, SB, BB, UTG, HJ, CO
/// - 9/10-max: BTN, SB, BB, UTG(0..n), MP(0..n), HJ, CO
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionName {
    BTN,
    SB,
    BB,
    UTG(usize),
    MP(usize),
    HJ,
    CO,
}
impl PositionName {
    /// Computes the position name for a seat relative to the dealer.
    pub fn from_seat(seat: usize, dealer: usize, table: usize) -> Self {
        let offset = (seat + table - dealer) % table;
        match table {
            2 => match offset {
                0 => Self::BTN,
                _ => Self::BB,
            },
            6 => match offset {
                0 => Self::BTN,
                1 => Self::SB,
                2 => Self::BB,
                3 => Self::UTG(0),
                4 => Self::HJ,
                _ => Self::CO,
            },
            _ => match offset {
                0 => Self::BTN,
                1 => Self::SB,
                2 => Self::BB,
                3 => Self::UTG(0),
                4 => Self::UTG(1),
                5 => Self::MP(0),
                6 => Self::MP(1),
                7 => Self::HJ,
                _ => Self::CO,
            },
        }
    }
}
impl std::fmt::Display for PositionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BTN => write!(f, "BTN"),
            Self::SB => write!(f, "SB"),
            Self::BB => write!(f, "BB"),
            Self::UTG(0) => write!(f, "UTG"),
            Self::UTG(n) => write!(f, "UTG+{n}"),
            Self::MP(0) => write!(f, "MP"),
            Self::MP(n) => write!(f, "MP+{n}"),
            Self::HJ => write!(f, "HJ"),
            Self::CO => write!(f, "CO"),
        }
    }
}
