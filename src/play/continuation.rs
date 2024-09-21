use crate::cards::street::Street;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Continuation {
    Decision(usize),
    Awaiting(Street),
    Terminal,
}
