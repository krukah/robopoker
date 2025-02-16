#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Next {
    Terminal,
    Chance,
    Choice(usize),
}

impl std::fmt::Display for Next {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Choice(c) => write!(f, "P{}", c),
            Self::Terminal => write!(f, "XX"),
            Self::Chance => write!(f, "??"),
        }
    }
}
