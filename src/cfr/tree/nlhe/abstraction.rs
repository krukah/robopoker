#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct Abstraction(u64);

impl Abstraction {
    pub fn new(i: u64) -> Self {
        Self(i)
    }
}
