#[derive(Debug, Default, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Ply {
    #[default]
    Terminal,
    Chance,
    Choice(usize),
}

impl std::fmt::Display for Ply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Choice(c) => write!(f, "P{}", c),
            Self::Terminal => write!(f, "XX"),
            Self::Chance => write!(f, "??"),
        }
    }
}
