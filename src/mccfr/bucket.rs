use super::path::Path;
use crate::clustering::abstraction::Abstraction;
use std::hash::Hash;

/// the product of
/// "information abstraction" and
/// "action absraction" are what we index the (regret, strategy, average, ...) on
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Bucket(pub Path, pub Abstraction);

impl From<(Path, Abstraction)> for Bucket {
    fn from((path, abstraction): (Path, Abstraction)) -> Self {
        Self(path, abstraction)
    }
}

impl From<&Bucket> for super::player::Player {
    fn from(bucket: &Bucket) -> Self {
        todo!("player should be stored or derviable from this. depth is_even, or we just store it, or some bitwise derviation idk yet")
    }
}

impl std::fmt::Display for Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.0, self.1)
    }
}
