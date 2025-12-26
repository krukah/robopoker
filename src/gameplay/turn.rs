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
    pub fn is_choice(&self) -> bool {
        matches!(self, Self::Choice(_))
    }
    pub fn is_chance(&self) -> bool {
        matches!(self, Self::Chance)
    }
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal)
    }
    pub fn display(&self) -> usize {
        match self {
            Self::Choice(c) => *c + 1,
            _ => panic!("don't ask"),
        }
    }
    pub fn label(&self) -> String {
        format!("P{}", self.display())
    }
}

impl crate::Arbitrary for Turn {
    fn random() -> Self {
        Self::Choice(rand::random_range(0..crate::N))
    }
}

impl crate::mccfr::TreeTurn for Turn {
    fn chance() -> Self {
        Self::Chance
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
