use rbp_mccfr::*;
use rbp_transport::Support;

/// Player, chance, or terminal indicator for Leduc Hold'em.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LeducTurn {
    Player(usize),
    Chance,
    Terminal,
}

impl From<usize> for LeducTurn {
    fn from(player: usize) -> Self {
        match player {
            0 => Self::Player(0),
            1 => Self::Player(1),
            _ => panic!("Leduc only has 2 players"),
        }
    }
}

impl Support for LeducTurn {}
impl CfrTurn for LeducTurn {
    fn chance() -> Self {
        Self::Chance
    }

    fn terminal() -> Self {
        Self::Terminal
    }

    fn players() -> usize {
        2
    }
}

impl std::fmt::Display for LeducTurn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chance => write!(f, "CH"),
            Self::Terminal => write!(f, "$$"),
            Self::Player(n) => write!(f, "P{n}"),
        }
    }
}
