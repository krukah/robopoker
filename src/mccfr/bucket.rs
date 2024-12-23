use super::path::Path;
use crate::clustering::abstraction::Abstraction;
use crate::Arbitrary;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(pub Path, pub Abstraction, pub Path);

impl From<(Path, Abstraction, Path)> for Bucket {
    fn from((past, present, future): (Path, Abstraction, Path)) -> Self {
        Self(past, present, future)
    }
}

impl std::fmt::Display for Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}::{}", self.0, self.1, self.2)
    }
}

impl Arbitrary for Bucket {
    fn random() -> Self {
        Self::from((Path::random(), Abstraction::random(), Path::random()))
    }
}
