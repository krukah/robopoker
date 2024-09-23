use crate::cards::street::Street;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Transition {
    Decision(usize),
    Awaiting(Street),
    Terminal,
}
