use crate::clustering::explorer::Abstraction;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(Abstraction);
impl Bucket {
    pub fn root() -> Bucket {
        return todo!("what is the bucket of the root node, what evn is the root node??");
    }
}

impl From<Abstraction> for Bucket {
    fn from(abstraction: Abstraction) -> Self {
        Self(abstraction)
    }
}
