use crate::cards::street::Street;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Transition {
    Choice(usize),
    Chance(Street),
    Terminal,
}

impl std::fmt::Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
