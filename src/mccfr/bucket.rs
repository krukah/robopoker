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
