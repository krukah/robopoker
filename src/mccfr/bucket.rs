use crate::clustering::abstraction::Abstraction;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Path(u64);

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(Path, Abstraction);
impl Bucket {
    pub fn root() -> Bucket {
        return todo!("what is the bucket of the root node, what evn is the root node??");
    }
}

impl From<(Path, Abstraction)> for Bucket {
    fn from((path, abstraction): (Path, Abstraction)) -> Self {
        Self(path, abstraction)
    }
}
