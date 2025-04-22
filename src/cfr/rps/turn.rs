#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Turn {
    P1,
    P2,
    Terminal,
}

impl crate::cfr::traits::turn::Turn for Turn {}
