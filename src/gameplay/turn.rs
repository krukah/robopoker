#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Turn {
    Terminal,
    Chance,
    Choice(usize),
}

impl Turn {
    pub fn position(&self) -> usize {
        match self {
            Self::Choice(c) => *c,
            _ => panic!("don't ask"),
        }
    }
}

impl crate::cfr::traits::turn::Turn for Turn {}

impl std::fmt::Display for Turn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Choice(c) => write!(f, "P{}", c),
            Self::Terminal => write!(f, "XX"),
            Self::Chance => write!(f, "??"),
        }
    }
}

impl TryFrom<&str> for Turn {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "XX" => Ok(Self::Terminal),
            "??" => Ok(Self::Chance),
            turn => {
                if turn.starts_with('P') {
                    turn[1..]
                        .parse::<usize>()
                        .map(Self::Choice)
                        .map_err(|_| "invalid player turn")
                } else {
                    Err("invalid ply input")
                }
            }
        }
    }
}
