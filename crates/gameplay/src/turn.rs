/// Whose turn it is to act in the game tree.
///
/// Distinguishes between player decision nodes, chance nodes (card deals),
/// and terminal nodes (hand complete).
///
/// # Variants
///
/// - `Choice(usize)` — Player `usize` must make a decision
/// - `Chance` — Dealer reveals cards (no player decision)
/// - `Terminal` — Hand is over, compute payoffs
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Turn {
    Terminal,
    Chance,
    Choice(usize),
}

impl Turn {
    /// Extracts the player index. Panics if not a Choice.
    pub fn position(&self) -> usize {
        match self {
            Self::Choice(c) => *c,
            _ => panic!("don't ask"),
        }
    }
    /// True if this is a player decision node.
    pub fn is_choice(&self) -> bool {
        matches!(self, Self::Choice(_))
    }
    /// True if this is a card deal node.
    pub fn is_chance(&self) -> bool {
        matches!(self, Self::Chance)
    }
    /// True if the hand is complete.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal)
    }
    /// 1-indexed player number for display.
    pub fn display(&self) -> usize {
        match self {
            Self::Choice(c) => *c + 1,
            _ => panic!("don't ask"),
        }
    }
    /// Display label (e.g., "P1", "P2").
    pub fn label(&self) -> String {
        format!("P{}", self.display())
    }
}

impl rbp_core::Arbitrary for Turn {
    fn random() -> Self {
        Self::Choice(rand::random_range(0..rbp_core::N))
    }
}

impl From<usize> for Turn {
    fn from(player: usize) -> Self {
        Self::Choice(player)
    }
}


impl TryFrom<&str> for Turn {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "XX" => Ok(Self::Terminal),
            "??" => Ok(Self::Chance),
            turn => turn[1..]
                .parse::<usize>()
                .map(Self::Choice)
                .map_err(|_| anyhow::anyhow!("invalid player turn")),
        }
    }
}

impl std::fmt::Display for Turn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Choice(c) => write!(f, "P{}", c),
            Self::Terminal => write!(f, "-"),
            Self::Chance => write!(f, "?"),
        }
    }
}

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
            Self::UTG(n) => write!(f, "UTG+{}", n),
            Self::MP(0) => write!(f, "MP"),
            Self::MP(n) => write!(f, "MP+{}", n),
            Self::HJ => write!(f, "HJ"),
            Self::CO => write!(f, "CO"),
        }
    }
}
