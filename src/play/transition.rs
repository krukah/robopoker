use crate::cards::street::Street;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Transition {
    Choice(usize),
    Chance(Street),
    Terminal,
}
