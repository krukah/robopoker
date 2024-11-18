#[derive(PartialEq)]
pub enum Phase {
    Discount,
    Explore,
    Prune,
}

impl From<usize> for Phase {
    fn from(epochs: usize) -> Self {
        match epochs {
            e if e < crate::CFR_DISCOUNT_PHASE => Phase::Discount,
            e if e < crate::CFR_PRUNNING_PHASE => Phase::Explore,
            _ => Phase::Prune,
        }
    }
}
